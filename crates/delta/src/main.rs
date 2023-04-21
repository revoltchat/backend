#[macro_use]
extern crate rocket;
#[macro_use]
extern crate revolt_rocket_okapi;
#[macro_use]
extern crate serde_json;

pub mod routes;
pub mod util;

use async_std::channel::unbounded;
use revolt_quark::authifier::{Authifier, AuthifierEvent};
use revolt_quark::events::client::EventV1;
use revolt_quark::DatabaseInfo;

#[launch]
async fn rocket() -> _ {
    // Configure logging and environment
    revolt_quark::configure!();

    // Ensure environment variables are present
    revolt_quark::variables::delta::preflight_checks();

    // Setup database
    let db = revolt_database::DatabaseInfo::Auto.connect().await.unwrap();
    let db = revolt_models::Database(db);
    db.migrate_database().await.unwrap();

    // Legacy database setup from quark
    let legacy_db = DatabaseInfo::Auto.connect().await.unwrap();

    // Setup Authifier event channel
    let (sender, receiver) = unbounded();

    // Setup Authifier
    let authifier = Authifier {
        database: legacy_db.clone().into(),
        config: revolt_quark::util::authifier::config(),
        event_channel: Some(sender),
    };

    // Launch a listener for Authifier events
    async_std::task::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match &event {
                AuthifierEvent::CreateSession { .. } | AuthifierEvent::CreateAccount { .. } => {
                    EventV1::Auth(event).global().await
                }
                AuthifierEvent::DeleteSession { user_id, .. }
                | AuthifierEvent::DeleteAllSessions { user_id, .. } => {
                    let id = user_id.to_string();
                    EventV1::Auth(event).private(id).await
                }
            }
        }
    });

    // Launch background task workers
    async_std::task::spawn(revolt_quark::tasks::start_workers(legacy_db.clone()));

    // Configure CORS
    let cors = revolt_quark::web::cors::new();

    // Configure Rocket
    let rocket = rocket::build();
    routes::mount(rocket)
        .mount("/", revolt_quark::web::cors::catch_all_options_routes())
        .mount("/", revolt_quark::web::ratelimiter::routes())
        .mount("/swagger/", revolt_quark::web::swagger::routes())
        .manage(authifier)
        .manage(db)
        .manage(legacy_db)
        .manage(cors.clone())
        .attach(revolt_quark::web::ratelimiter::RatelimitFairing)
        .attach(cors)
}
