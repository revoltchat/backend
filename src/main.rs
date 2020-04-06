#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

pub mod database;
pub mod email;
pub mod guards;
pub mod routes;
pub mod websocket;

use dotenv;
use rocket_cors::AllowedOrigins;
use std::thread;

fn main() {
    dotenv::dotenv().ok();
    database::connect();

    thread::spawn(|| {
        websocket::launch_server();
    });

    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::All,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    routes::mount(rocket::ignite()).attach(cors).launch();
}
