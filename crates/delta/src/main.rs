#[macro_use]
extern crate rocket;
#[macro_use]
extern crate revolt_rocket_okapi;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

pub mod routes;
pub mod util;

use async_std::channel::unbounded;
use revolt_quark::events::client::EventV1;
use revolt_quark::rauth::{RAuth, RAuthEvent};
use revolt_quark::DatabaseInfo;
use rocket::data::ToByteUnit;

#[launch]
async fn rocket() -> _ {
    // Configure logging and environment
    revolt_quark::configure!();

    // Ensure environment variables are present
    revolt_quark::variables::delta::preflight_checks();

    // Setup database
    let db = DatabaseInfo::Auto.connect().await.unwrap();
    db.migrate_database().await.unwrap();

    // Setup rAuth event channel
    let (sender, receiver) = unbounded();

    // Setup rAuth
    let rauth = RAuth {
        database: db.clone().into(),
        config: revolt_quark::util::rauth::config(),
        event_channel: Some(sender),
    };

    // Launch a listener for rAuth events
    async_std::task::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match &event {
                RAuthEvent::CreateSession { .. } | RAuthEvent::CreateAccount { .. } => {
                    EventV1::Auth(event).global().await
                }
                RAuthEvent::DeleteSession { user_id, .. }
                | RAuthEvent::DeleteAllSessions { user_id, .. } => {
                    let id = user_id.to_string();
                    EventV1::Auth(event).private(id).await
                }
            }
        }
    });

    // Launch background task workers
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
        .configure(rocket::Config {
            limits: rocket::data::Limits::default()
                .limit("string", 5.megabytes()),
            ..Default::default()
        })
}
