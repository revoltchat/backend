#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate ctrlc;

//pub mod database;
//pub mod notifications;
pub mod routes;
//pub mod redis;
pub mod util;
pub mod version;
//pub mod task_queue;

use async_std::task;
use futures::join;
use log::info;
use rauth::{
    config::{Captcha, Config, EmailVerification, SMTPSettings, Template, Templates},
    logic::Auth,
};
use revolt_quark::DatabaseInfo;
use rocket_cors::AllowedOrigins;
use std::str::FromStr;
use util::variables::{
    APP_URL, HCAPTCHA_KEY, INVITE_ONLY, SMTP_FROM, SMTP_HOST, SMTP_PASSWORD, SMTP_USERNAME,
    USE_EMAIL, USE_HCAPTCHA,
};
// use crate::util::ratelimit::RatelimitState;

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!(
        "Starting REVOLT server [version {}].",
        crate::version::VERSION
    );

    /*util::variables::preflight_checks();
    database::connect().await;
    redis::connect().await;
    notifications::hive::init_hive().await;
    task_queue::start_queues().await;*/

    ctrlc::set_handler(move || {
        // Force ungraceful exit to avoid hang.
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let web_task = task::spawn(launch_web());
    //let hive_task = task::spawn_local(notifications::hive::listen());

    join!(
        web_task,
        //hive_task,
        //notifications::websocket::launch_server()
    );
}

async fn launch_web() {
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
                        text: include_str!("../assets/templates/verify.txt").into(),
                        url: format!("{}/login/verify/", *APP_URL),
                        html: None,
                    },
                    reset: Template {
                        title: "Reset your Revolt password.".into(),
                        text: include_str!("../assets/templates/reset.txt").into(),
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

    let mongo_db = mongodb::Client::with_uri_str("mongodb://localhost")
        .await
        .expect("Failed to init db connection.");

    rauth::entities::sync_models(&mongo_db.database("revolt")).await;

    let auth = Auth::new(mongo_db.database("revolt"), config);
    let rocket = rocket::build();
    routes::mount(rocket)
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/auth/account", rauth::web::account::routes())
        .mount("/auth/session", rauth::web::session::routes())
        .manage(auth)
        .manage(db)
        .manage(cors.clone())
        //.manage(RatelimitState::new())
        .attach(cors)
        .launch()
        .await
        .unwrap();
}
