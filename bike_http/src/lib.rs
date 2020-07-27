#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::str::FromStr;
use std::sync::Arc;

use bike_core::{Bicycle, Color, Repository};
use rocket::State;
use rocket::config::{Config, Environment};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::bike_api::BicycleApi;

mod bike_api;

#[derive(Serialize, Deserialize, Debug)]
struct BicycleDto {
    id: Option<i64>,
    model: String,
    color: String,
}

#[get("/")]
fn index() -> &'static str {
    "Bike Inventory!"
}

#[get("/<id>")]
fn get_bike(id: i64, api: State<BicycleApi>) -> Option<Json<BicycleDto>> {
    api.get_bike(id)
        .map(|bike| Json(bike.into()))
        .ok()
}

#[post("/", format = "json", data = "<json_bike>")]
fn create_bike(json_bike: Json<BicycleDto>, api: State<BicycleApi>) -> Option<Json<BicycleDto>> {
    let bike = json_bike.into_inner();
    api.create_bike(bike.into())
        .map(|bike| Json(bike.into()))
        .ok()
}

#[put("/<id>", format = "json", data = "<json_bike>")]
fn update_bike(id: i64, json_bike: Json<BicycleDto>, api: State<BicycleApi>) -> Option<Json<BicycleDto>> {
    let mut bike = json_bike.into_inner();
    bike.id = Some(id);
    api.update_bike(bike.into())
        .map(|bike| Json(bike.into()))
        .ok()
}

pub fn initialize(repo: &Arc<Box<dyn Repository>>) {
    let api = BicycleApi::new(repo.clone());

    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8000)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .manage(api)
        .mount("/", routes![index])
        .mount("/bike", routes![create_bike, update_bike, get_bike])
        .launch();
}

impl From<BicycleDto> for Bicycle {
    fn from(b: BicycleDto) -> Self {
        let id = b.id.unwrap_or(0);
        let color = Color::from_str(b.color.as_str()).expect("Unexpected Color");
        Bicycle {
            id,
            model: b.model.clone(),
            color,
        }
    }
}

impl From<Bicycle> for BicycleDto {
    fn from(b: Bicycle) -> Self {
        BicycleDto {
            id: Some(b.id),
            model: b.model.clone(),
            color: b.color.to_string(),
        }
    }
}
