use strum;
use strum_macros::{Display, EnumString};

#[derive(Debug, Display, EnumString, Clone)]
pub enum Color {
    Blue,
    Red,
    White,
    Black,
    Gray,
}

#[derive(Debug, Clone)]
pub struct Bicycle {
    pub id: i64,
    pub model: String,
    pub color: Color,
}

pub type RepositoryResult<T> = core::result::Result<T, RepositoryError>;

pub trait Repository: Send + Sync {
    fn get(&self, id: i64) -> RepositoryResult<Bicycle>;

    fn insert(&self, bike: &Bicycle) -> RepositoryResult<Bicycle>;

    fn update(&self, bike: &Bicycle) -> RepositoryResult<Bicycle>;

    fn get_all(&self, page: i64, limit: i64) -> RepositoryResult<Vec<Bicycle>>;
}

#[derive(Debug)]
pub enum RepositoryError {
    ConnectionError(String),
    NotFound,
    StorageError(String),
    IdDoesntExist,
    OperationCancelled,
}
