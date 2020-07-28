use std::sync::Arc;

use bike_core::Repository;
use bike_repo::mysql_repo::MySqlBicycleRepo;

fn main() {
    let repo = MySqlBicycleRepo::new();
    let repo_shared: Arc<dyn Repository> = Arc::new(repo);

    bike_http::initialize(&repo_shared);
    // grpc::initialize(&repo_shared);
}
