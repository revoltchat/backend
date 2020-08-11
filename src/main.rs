#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate bitfield;
#[macro_use]
extern crate lazy_static;

pub mod notifications;
pub mod database;
pub mod routes;
pub mod email;
pub mod util;

use dotenv;
use rocket_cors::AllowedOrigins;
use std::thread;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    database::connect();
    notifications::start_worker();

    thread::spawn(|| {
        notifications::pubsub::launch_subscriber();
    });

    thread::spawn(|| {
        notifications::ws::launch_server();
    });

    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::All,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    routes::mount(rocket::ignite()).attach(cors).launch();
}
