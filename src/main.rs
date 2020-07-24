use bike_core::Repository;
use bike_repo::mysql_repo::MySqlBicycleRepo;

fn main() {
    let repo = MySqlBicycleRepo::new();
    //
    // let id = 1;
    // match repo.get(1) {
    //     Ok(bike) => println!("Bike with id {} found: {:?}", id, bike),
    //     Err(_err) => println!("Bike with id {} not found", id),
    // }

    println!("Hello, world!");
}
