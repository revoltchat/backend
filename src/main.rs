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
pub mod version;
pub mod routes;
pub mod util;

use async_std::task;
use chrono::Duration;
use futures::join;
use log::info;
use rauth::options::{EmailVerification, Options, SMTP};
use rauth::{
    auth::Auth,
    options::{Template, Templates},
};
use rocket_cors::AllowedOrigins;
use rocket_prometheus::PrometheusMetrics;
use util::variables::{
    APP_URL, HCAPTCHA_KEY, INVITE_ONLY, PUBLIC_URL, SMTP_FROM, SMTP_HOST, SMTP_PASSWORD,
    SMTP_USERNAME, USE_EMAIL, USE_HCAPTCHA, USE_PROMETHEUS,
};

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!("Starting REVOLT server [version {}].", crate::version::VERSION);

    util::variables::preflight_checks();
    database::connect().await;
    notifications::hive::init_hive().await;

    ctrlc::set_handler(move || {
        // Force ungraceful exit to avoid hang.
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let web_task = task::spawn(launch_web());
    let hive_task = task::spawn(notifications::hive::listen());

    join!(
        web_task,
        hive_task,
        notifications::websocket::launch_server()
    );
}

async fn launch_web() {
    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::All,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS.");

    let mut options = Options::new()
        .base_url(format!("{}/auth", *PUBLIC_URL))
        .email_verification(if *USE_EMAIL {
            EmailVerification::Enabled {
                success_redirect_uri: format!("{}/login", *APP_URL),
                welcome_redirect_uri: format!("{}/welcome", *APP_URL),
                password_reset_url: Some(format!("{}/login/reset", *APP_URL)),

                verification_expiry: Duration::days(1),
                password_reset_expiry: Duration::hours(1),

                templates: Templates {
                    verify_email: Template {
                        title: "Verify your REVOLT account.",
                        text: "Verify your email here: {{url}}",
                        html: include_str!("../assets/templates/verify.html"),
                    },
                    reset_password: Template {
                        title: "Reset your REVOLT password.",
                        text: "Reset your password here: {{url}}",
                        html: include_str!("../assets/templates/reset.html"),
                    },
                    welcome: None,
                },

                smtp: SMTP {
                    from: (*SMTP_FROM).to_string(),
                    host: (*SMTP_HOST).to_string(),
                    username: (*SMTP_USERNAME).to_string(),
                    password: (*SMTP_PASSWORD).to_string(),
                },
            }
        } else {
            EmailVerification::Disabled
        });

    if *INVITE_ONLY {
        options = options.invite_only_collection(database::get_collection("invites"))
    }

    if *USE_HCAPTCHA {
        options = options.hcaptcha_secret(HCAPTCHA_KEY.clone());
    }

    let auth = Auth::new(database::get_collection("accounts"), options);

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
