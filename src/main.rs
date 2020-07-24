use bike_core::{Bicycle, Color, Repository};
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
        color: Color::Blue,
    };
    let new_bike = repo.insert(&b);
    match &new_bike {
        Ok(bike) => println!("Bike with id {} found: {:?}", bike.id, bike),
        Err(err) => println!("Created bike not found: {:?}", err),
    }

    for b in repo.get_all(1, 2).unwrap() {
        println!("Bike: {:?}", b);
    }

    let r = repo.save(Some(7), |o_bike| {
        o_bike.map(|b|
            {
                let mut new_model = b.model.clone();
                new_model.push_str(" (Updated!)");
                Bicycle {
                    id: b.id,
                    model: new_model,
                    color: Color::Gray,
                }
            }
        )
    });

    println!("----------------------------");
    match &r {
        Ok(bike) => println!("Bike updated id {}: {:?}", bike.id, bike),
        Err(err) => println!("Error updating bike: {:?}", err),
    }

    println!("Hello, world!");
}
