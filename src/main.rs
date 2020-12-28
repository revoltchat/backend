#![feature(proc_macro_hygiene, decl_macro)]
#![feature(async_closure)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
extern crate ctrlc;

pub mod notifications;
pub mod database;
pub mod routes;
pub mod util;

use rauth;
use log::info;
use futures::join;
use async_std::task;
use rocket_cors::AllowedOrigins;

fn main() {
    task::block_on(entry())
}

async fn entry() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!("Starting REVOLT server.");

    util::variables::preflight_checks();
    database::connect().await;

    ctrlc::set_handler(move || {
        // Force ungraceful exit to avoid hang.
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
    
    join!(launch_web(), notifications::websocket::launch_server());
}

async fn launch_web() {
    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::All,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS.");
    
    let auth = rauth::auth::Auth::new(database::get_collection("accounts"));

    routes::mount(rauth::routes::mount(rocket::ignite(), "/auth", auth))
        .mount("/", rocket_cors::catch_all_options_routes())
        .manage(cors.clone())
        .attach(cors)
        .launch()
        .await
        .unwrap();
}
