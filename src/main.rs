#![feature(proc_macro_hygiene, decl_macro)]
#![feature(async_closure)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate bitfield;
#[macro_use]
extern crate lazy_static;

pub mod database;
pub mod pubsub;
pub mod routes;
pub mod util;

use log::info;
use rocket_cors::AllowedOrigins;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!("Starting REVOLT server.");

    util::variables::preflight_checks();
    database::connect().await;

    pubsub::hive::init_hive();
    //pubsub::websocket::launch_server();

    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::All,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS.");

    routes::mount(rocket::ignite())
        .mount("/", rocket_cors::catch_all_options_routes())
        .manage(cors.clone())
        .attach(cors)
        .launch()
        .await
        .unwrap();
}
