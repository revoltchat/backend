#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

pub mod routes;
pub mod util;

use revolt_quark::rauth::RAuth;
use revolt_quark::DatabaseInfo;

#[launch]
async fn rocket() -> _ {
    // Configure logging and environment
    revolt_quark::configure!();

    // Ensure environment variables are present
    revolt_quark::variables::delta::preflight_checks();

    // Setup database
    let db = DatabaseInfo::Auto.connect().await.unwrap();
    db.migrate_database().await.unwrap();

    // Setup rAuth
    let rauth = RAuth {
        database: db.clone().into(),
        config: revolt_quark::util::rauth::config(),
    };

    // Launch background task workers.
    async_std::task::spawn(revolt_quark::tasks::start_workers(db.clone()));

    // Configure CORS
    let cors = revolt_quark::web::cors::new();

    // Configure Rocket
    let rocket = rocket::build();
    routes::mount(rocket)
        .mount("/", revolt_quark::web::cors::catch_all_options_routes())
        .mount("/", revolt_quark::web::ratelimiter::routes())
        .mount("/swagger/", revolt_quark::web::swagger::routes())
        .manage(rauth)
        .manage(db)
        .manage(cors.clone())
        .attach(revolt_quark::web::ratelimiter::RatelimitFairing)
        .attach(cors)
}
