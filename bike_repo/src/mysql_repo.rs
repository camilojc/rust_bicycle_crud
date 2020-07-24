use core::convert;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

use bike_core::{Color, Repository, RepositoryError, RepositoryResult};
use bike_core::Bicycle as CoreBicycle;
use mysql::{Opts, OptsBuilder, Transaction};
use mysql::prelude::Queryable;
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MysqlConnectionManager;

type Connection = PooledConnection<MysqlConnectionManager>;

pub struct MySqlBicycleRepo {
    pool: Arc<Pool<MysqlConnectionManager>>,
}

#[derive(Clone)]
struct Bicycle {
    bike: CoreBicycle,
    created_at: i64,
    updated_at: i64,
    version: i64,
}

impl MySqlBicycleRepo {
    pub fn new() -> MySqlBicycleRepo {
        let url = env::var("DATABASE_URL").expect("The DATABASE_URL env variable is required!");
        let opts = Opts::from_url(&url).unwrap();
        let builder = OptsBuilder::from_opts(opts);
        let manager = MysqlConnectionManager::new(builder);
        let conn_pool = r2d2::Pool::builder()
            .min_idle(Some(2))
            .max_size(4)
            .build(manager);
        match &conn_pool {
            Ok(_) => println!("Connected successfully to the DB"),
            Err(e) => println!("Error connecting to DB: {}", e.to_string())
        }
        let pool = Arc::new(conn_pool.unwrap());
        MySqlBicycleRepo {
            pool
        }
    }

    fn get_connection(&self) -> Result<Connection, RepositoryError> {
        self.pool.get().map_err(|err| RepositoryError::ConnectionError(err.to_string()))
    }

    fn start_tx<'a, 'b>(&'a self, conn: &'b mut Connection) -> Result<Transaction, RepositoryError>
        where 'b : 'a {
        let tx_opts = mysql::TxOpts::default()
            .set_isolation_level(Some(mysql::IsolationLevel::Serializable));
        conn.start_transaction(tx_opts)
            .map_err(MySqlBicycleRepo::mysql_error_mapper)
    }

    fn sql_bicycle_mapper() -> fn((i64, String, String, i64, i64, i64)) -> Bicycle {
        |(b_id, model, color, created_at, updated_at, version): (i64, String, String, i64, i64, i64)| {
            Bicycle {
                bike: CoreBicycle {
                    id: b_id,
                    model: String::from(model),
                    color: Color::from_str(color.as_str()).unwrap(),
                },
                created_at,
                updated_at,
                version,
            }
        }
    }

    fn mysql_error_mapper(err: mysql::Error) -> RepositoryError {
        RepositoryError::StorageError(err.to_string())
    }

    fn get_for_update(&self, tx: &mut Transaction, id: i64) -> RepositoryResult<Bicycle> {
        let query = tx.prep(GET_FOR_UPDATE).map_err(MySqlBicycleRepo::mysql_error_mapper)?;
        tx.exec_map(&query, (id, ), MySqlBicycleRepo::sql_bicycle_mapper())
            .map_err(|err| RepositoryError::StorageError(err.to_string()))
            .map(|bikes| {
                if bikes.len() == 0 {
                    Err(RepositoryError::IdDoesntExist)
                } else {
                    Ok(bikes[0].clone())
                }
            })
            .and_then(convert::identity)
    }

    fn insert_in_tx(&self, mut tx: Transaction, bike: &CoreBicycle) -> RepositoryResult<CoreBicycle> {
        let query = tx.prep(INSERT_QUERY).map_err(MySqlBicycleRepo::mysql_error_mapper)?;
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        tx.exec_drop(&query, (&bike.model, bike.color.to_string(), now, now))
            .map_err(MySqlBicycleRepo::mysql_error_mapper)
            .map(|()| tx.last_insert_id())
            .map(|oid| {
                match oid {
                    Some(id) => {
                        println!("last insert id: {}", id);
                        self.get_in_tx(&mut tx, id as i64)
                    }
                    None => Err(RepositoryError::IdDoesntExist)
                }
            })
            .and_then(|val| {
                tx.commit().map_err(MySqlBicycleRepo::mysql_error_mapper)?;
                val
            })
    }

    fn update_in_tx(&self, mut tx: Transaction, bike: &CoreBicycle) -> Result<CoreBicycle, RepositoryError> {
        let query = tx.prep(UPDATE_QUERY).map_err(MySqlBicycleRepo::mysql_error_mapper)?;
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        self.get_for_update(&mut tx, bike.id)
            .and_then(|b| {
                tx.exec_drop(query, (&bike.model, bike.color.to_string(), now, b.version + 1, bike.id))
                    .map_err(MySqlBicycleRepo::mysql_error_mapper)
            })
            .and_then(|()| {
                tx.commit().map_err(MySqlBicycleRepo::mysql_error_mapper)?;
                self.get(bike.id)
            })
    }

    fn get_in_tx(&self, tx: &mut Transaction, id: i64) -> RepositoryResult<CoreBicycle> {
        let query = tx.prep(GET_QUERY).map_err(MySqlBicycleRepo::mysql_error_mapper)?;
        tx.exec_map(&query, (id, ), MySqlBicycleRepo::sql_bicycle_mapper())
            .map_err(|err| RepositoryError::StorageError(err.to_string()))
            .map(|bikes| {
                if bikes.len() == 0 {
                    Err(RepositoryError::IdDoesntExist)
                } else {
                    Ok(bikes[0].bike.clone())
                }
            })
            .and_then(convert::identity)
    }
}

impl Repository for MySqlBicycleRepo {
    fn get(&self, id: i64) -> RepositoryResult<CoreBicycle> {
        let mut conn = self.get_connection()?;
        let query = conn.prep(GET_QUERY).map_err(MySqlBicycleRepo::mysql_error_mapper)?;
        conn.exec_map(&query, (id, ), MySqlBicycleRepo::sql_bicycle_mapper())
            .map_err(|err| RepositoryError::StorageError(err.to_string()))
            .map(|bikes| {
                if bikes.len() == 0 {
                    Err(RepositoryError::IdDoesntExist)
                } else {
                    Ok(bikes[0].bike.clone())
                }
            })
            .and_then(convert::identity)
    }

    fn insert(&self, bike: &CoreBicycle) -> RepositoryResult<CoreBicycle> {
        let mut conn = self.get_connection()?;
        let tx = self.start_tx(&mut conn)?;
        self.insert_in_tx(tx, bike)
    }

    fn update(&self, bike: &CoreBicycle) -> RepositoryResult<CoreBicycle> {
        let mut conn = self.get_connection()?;
        let tx = self.start_tx(&mut conn)?;
        self.update_in_tx(tx, bike)
    }

    fn save<F>(&self, oid: Option<i64>, f: F) -> RepositoryResult<CoreBicycle>
        where F: FnOnce(Option<CoreBicycle>) -> CoreBicycle {
        let mut conn = self.get_connection()?;
        let mut tx = self.start_tx(&mut conn)?;
        return if let Some(id) = oid {
            self.get_for_update(&mut tx, id)
                .map(|bike| f(Some(bike.bike.clone())))
                .and_then(|bike| self.update_in_tx(tx, &bike))
        } else {
            let bike_to_insert = f(None);
            self.insert_in_tx(tx, &bike_to_insert)
        };
    }

    fn get_all(&self, page: i64, limit: i64) -> RepositoryResult<Vec<CoreBicycle>> {
        let mut conn = self.get_connection()?;
        let query = conn.prep(GET_ALL_QUERY).map_err(MySqlBicycleRepo::mysql_error_mapper)?;
        conn.exec_map(&query, (limit, limit * page, ), MySqlBicycleRepo::sql_bicycle_mapper())
            .map_err(|err| RepositoryError::StorageError(err.to_string()))
            .map(|bikes| -> Vec<CoreBicycle> { bikes.into_iter().map(|b| b.bike).collect() })
    }
}


const GET_QUERY: &str = "SELECT b_id, model, color, created_at, updated_at, version FROM bicycle WHERE b_id = ?";
const GET_FOR_UPDATE: &str = "SELECT b_id, model, color, created_at, updated_at, version FROM bicycle WHERE b_id = ? FOR UPDATE";
const INSERT_QUERY: &str = "INSERT INTO bicycle(model, color, created_at, updated_at, version) VALUES (?, ?, ?, ?, 0)";
const UPDATE_QUERY: &str = "UPDATE bicycle SET model = ?, color = ?, updated_at = ?, version = ? WHERE b_id = ?";
const GET_ALL_QUERY: &str = "SELECT b_id, model, color, created_at, updated_at, version FROM bicycle ORDER BY b_id LIMIT ? OFFSET ?";
