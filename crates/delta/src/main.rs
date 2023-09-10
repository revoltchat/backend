#[macro_use]
extern crate rocket;
#[macro_use]
extern crate revolt_rocket_okapi;
#[macro_use]
extern crate serde_json;

pub mod routes;
pub mod util;

use revolt_database::{Database, MongoDb};
use rocket::{Build, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_prometheus::PrometheusMetrics;
use std::net::Ipv4Addr;
use std::str::FromStr;

use async_std::channel::unbounded;
use revolt_quark::authifier::{Authifier, AuthifierEvent};
use revolt_quark::events::client::EventV1;
use revolt_quark::DatabaseInfo;
use rocket::data::ToByteUnit;

pub async fn web() -> Rocket<Build> {
    // Setup database
    let db = revolt_database::DatabaseInfo::Auto.connect().await.unwrap();
    db.migrate_database().await.unwrap();

    // Legacy database setup from quark
    let legacy_db = DatabaseInfo::Auto.connect().await.unwrap();

    // Setup Authifier event channel
    let (sender, receiver) = unbounded();

    // Setup Authifier
    let authifier = Authifier {
        database: match db.clone() {
            Database::Reference(_) => Default::default(),
            Database::MongoDb(MongoDb(client, _)) => authifier::Database::MongoDb(
                authifier::database::MongoDb(client.database("revolt")),
            ),
        },
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
    async_std::task::spawn(revolt_database::tasks::start_workers(
        db.clone(),
        authifier.database.clone(),
    ));
    async_std::task::spawn(revolt_quark::tasks::start_workers(
        legacy_db.clone(),
        authifier.database.clone(),
    ));

    // Configure CORS
    let cors = CorsOptions {
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

    // Configure Swagger
    let swagger = revolt_rocket_okapi::swagger_ui::make_swagger_ui(
        &revolt_rocket_okapi::swagger_ui::SwaggerUIConfig {
            url: "../openapi.json".to_owned(),
            ..Default::default()
        },
    )
    .into();

    // Configure Rocket
    let rocket = rocket::build();
    let prometheus = PrometheusMetrics::new();

    routes::mount(rocket)
        .attach(prometheus.clone())
        .mount("/metrics", prometheus)
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/", util::ratelimiter::routes())
        .mount("/swagger/", swagger)
        .manage(authifier)
        .manage(db)
        .manage(legacy_db)
        .manage(cors.clone())
        .attach(util::ratelimiter::RatelimitFairing)
        .attach(cors)
        .configure(rocket::Config {
            limits: rocket::data::Limits::default().limit("string", 5.megabytes()),
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Default::default()
        })
}

#[launch]
async fn rocket() -> _ {
    // Configure logging and environment
    revolt_quark::configure!();

    // Ensure environment variables are present
    revolt_quark::variables::delta::preflight_checks();

    // Start web server
    web().await
}
