#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate bitfield;
#[macro_use]
extern crate lazy_static;

pub mod database;
pub mod notifications;
pub mod routes;
pub mod util;

use rocket_cors::AllowedOrigins;
use std::thread;
use log::info;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(
        env_logger::Env::default()
            .filter_or("RUST_LOG", "info")
    );

    info!("Starting REVOLT server.");

    util::variables::preflight_checks();
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
