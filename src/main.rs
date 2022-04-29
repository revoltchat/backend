#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate ctrlc;

pub mod routes;
pub mod util;
pub mod version;

use log::info;
use rauth::{
    config::{Captcha, Config, EmailVerification, SMTPSettings, Template, Templates},
    logic::Auth,
};
use revolt_quark::variables::delta::{
    APP_URL, HCAPTCHA_KEY, INVITE_ONLY, SMTP_FROM, SMTP_HOST, SMTP_PASSWORD, SMTP_USERNAME,
    USE_EMAIL, USE_HCAPTCHA,
};
use revolt_quark::DatabaseInfo;
use rocket_cors::AllowedOrigins;
use std::str::FromStr;

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!(
        "Starting Revolt server [version {}].",
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

    let mut config = Config {
        email_verification: if *USE_EMAIL {
            EmailVerification::Enabled {
                smtp: SMTPSettings {
                    from: (*SMTP_FROM).to_string(),
                    host: (*SMTP_HOST).to_string(),
                    username: (*SMTP_USERNAME).to_string(),
                    password: (*SMTP_PASSWORD).to_string(),
                    reply_to: Some("support@revolt.chat".into()),
                    port: None,
                    use_tls: None,
                },
                expiry: Default::default(),
                templates: Templates {
                    verify: Template {
                        title: "Verify your Revolt account.".into(),
                        text: include_str!(crate::asset!("templates/verify.txt")).into(),
                        url: format!("{}/login/verify/", *APP_URL),
                        html: None,
                    },
                    reset: Template {
                        title: "Reset your Revolt password.".into(),
                        text: include_str!(crate::asset!("templates/reset.txt")).into(),
                        url: format!("{}/login/reset/", *APP_URL),
                        html: None,
                    },
                    welcome: None,
                },
            }
        } else {
            EmailVerification::Disabled
        },
        ..Default::default()
    };

    if *INVITE_ONLY {
        config.invite_only = true;
    }

    if *USE_HCAPTCHA {
        config.captcha = Captcha::HCaptcha {
            secret: HCAPTCHA_KEY.clone(),
        };
    }

    let db = DatabaseInfo::Auto.connect().await.unwrap();
    db.migrate_database().await.unwrap();

    // This is entirely temporary code until rauth is migrated to quark.
    // (and / or otherwise gets updated to MongoDB v2 driver)
    let mongo_db = mongodb::Client::with_uri_str(
        &std::env::var("MONGODB").unwrap_or_else(|_| "mongodb://localhost".to_string()),
    )
    .await
    .expect("Failed to init db connection.");

    rauth::entities::sync_models(&mongo_db.database("revolt")).await;

    // Launch background task workers.
    async_std::task::spawn(revolt_quark::tasks::start_workers(db.clone()));

    let auth = Auth::new(mongo_db.database("revolt"), config);
    let rocket = rocket::build();
    routes::mount(rocket)
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/", util::ratelimiter::routes())
        .mount(
            "/swagger/",
            rocket_okapi::swagger_ui::make_swagger_ui(&rocket_okapi::swagger_ui::SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .manage(auth)
        .manage(db)
        .manage(cors.clone())
        .attach(util::ratelimiter::RatelimitFairing)
        .attach(cors)
        .launch()
        .await
        .unwrap();
}

/// Resolve asset
macro_rules! asset {
    ($path:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path)
    };
}

pub(crate) use asset;
