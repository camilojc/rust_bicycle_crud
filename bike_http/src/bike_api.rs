use std::sync::Arc;

use bike_core::{Bicycle, Repository, RepositoryResult};

pub struct BicycleApi {
    repo: Arc<dyn Repository>,
}

impl BicycleApi {
    pub fn new(repo: Arc<dyn Repository>) -> BicycleApi {
        BicycleApi {
            repo
        }
    }

    pub fn create_bike(&self, bike: Bicycle) -> RepositoryResult<Bicycle> {
        self.repo.insert(&bike)
    }

    pub fn update_bike(&self, bike: Bicycle) -> RepositoryResult<Bicycle> {
        self.repo.update(&bike)
    }

    pub fn get_bike(&self, id: i64) -> RepositoryResult<Bicycle> {
        self.repo.get(id)
    }
}
