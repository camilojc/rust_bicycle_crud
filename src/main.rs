use bike_core::{Repository, Bicycle, Color};
use bike_repo::mysql_repo::MySqlBicycleRepo;

fn main() {
    let repo = MySqlBicycleRepo::new();

    let id = 1;
    match repo.get(1) {
        Ok(bike) => println!("Bike with id {} found: {:?}", id, bike),
        Err(_err) => println!("Bike with id {} not found", id),
    }

    let b = Bicycle {
        id: 0,
        model: "Another Model".to_string(),
        color: Color::Blue
    };
    let new_bike = repo.insert(&b);
    match &new_bike {
        Ok(bike) => println!("Bike with id {} found: {:?}", bike.id, bike),
        Err(err) => println!("Created bike not found: {:?}", err),
    }

    println!("Hello, world!");
}
