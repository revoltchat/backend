#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
extern crate ctrlc;

pub mod routes;
pub mod version;

use log::info;
use rauth::{
    config::Config,
    logic::Auth,
};
use revolt_quark::DatabaseInfo;
use rocket_cors::AllowedOrigins;
use std::str::FromStr;

#[async_std::main]
async fn main() {
    let _guard = revolt_quark::setup_logging();

    info!(
        "Starting Revcord server [version {}].",
        crate::version::VERSION
    );

    revolt_quark::variables::delta::preflight_checks();

    #[cfg(debug_assertions)]
    ctrlc::set_handler(move || {
        // Force ungraceful exit to avoid hang.
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::All,
        allowed_methods: [
            "Get", "Put", "Post", "Delete", "Options", "Head", "Trace", "Connect", "Patch",
        ]
        .iter()
        .map(|s| FromStr::from_str(s).unwrap())
        .collect(),
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS.");

    let db = DatabaseInfo::Auto.connect().await.unwrap();

    // This is entirely temporary code until rauth is migrated to quark.
    // (and / or otherwise gets updated to MongoDB v2 driver)
    let mongo_db = mongodb::Client::with_uri_str(
        &std::env::var("MONGODB").unwrap_or_else(|_| "mongodb://localhost".to_string()),
    )
    .await
    .expect("Failed to init db connection.");

    // Launch background task workers.
    async_std::task::spawn(revolt_quark::tasks::start_workers(db.clone()));

    let auth = Auth::new(mongo_db.database("revolt"), Config::default());
    let rocket = rocket::build();
    routes::mount(rocket)
        .mount("/", rocket_cors::catch_all_options_routes())
        .manage(auth)
        .manage(db)
        .manage(cors.clone())
        .attach(cors)
        .launch()
        .await
        .unwrap();
}
