#![feature(proc_macro_hygiene, decl_macro)]
#![feature(async_closure)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate impl_ops;
#[macro_use]
extern crate bitfield;
extern crate ctrlc;

pub mod database;
pub mod notifications;
pub mod routes;
pub mod util;

use chrono::Duration;
use futures::join;
use log::info;
use rauth::auth::Auth;
use rauth::options::{EmailVerification, Options, SMTP};
use rocket_cors::AllowedOrigins;
use rocket_prometheus::PrometheusMetrics;
use util::variables::{
    PUBLIC_URL, SMTP_FROM, SMTP_HOST, SMTP_PASSWORD, SMTP_USERNAME, USE_EMAIL, USE_PROMETHEUS,
};

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!("Starting REVOLT server.");

    util::variables::preflight_checks();
    database::connect().await;
    notifications::hive::init_hive().await;

    ctrlc::set_handler(move || {
        // Force ungraceful exit to avoid hang.
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    join!(
        launch_web(),
        notifications::websocket::launch_server(),
        notifications::hive::listen(),
    );
}

async fn launch_web() {
    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::All,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS.");

    let auth = Auth::new(
        database::get_collection("accounts"),
        Options::new()
            .base_url(format!("{}/auth", *PUBLIC_URL))
            .email_verification(if *USE_EMAIL {
                EmailVerification::Enabled {
                    success_redirect_uri: format!("{}/welcome", *PUBLIC_URL),
                    verification_expiry: Duration::days(1),
                    verification_ratelimit: Duration::minutes(1),

                    smtp: SMTP {
                        from: (*SMTP_FROM).to_string(),
                        host: (*SMTP_HOST).to_string(),
                        username: (*SMTP_USERNAME).to_string(),
                        password: (*SMTP_PASSWORD).to_string(),
                    },
                }
            } else {
                EmailVerification::Disabled
            }),
    );

    let mut rocket = rocket::ignite();

    if *USE_PROMETHEUS {
        info!("Enabled Prometheus metrics!");
        let prometheus = PrometheusMetrics::new();
        rocket = rocket
            .attach(prometheus.clone())
            .mount("/metrics", prometheus);
    }

    routes::mount(rocket)
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/auth", rauth::routes::routes())
        .manage(auth)
        .manage(cors.clone())
        .attach(cors)
        .launch()
        .await
        .unwrap();
}
