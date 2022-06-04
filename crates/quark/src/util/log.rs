/// Configure logging and common Rust variables
pub fn setup_logging() -> sentry::ClientInitGuard {
    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    if std::env::var("ROCKET_ADDRESS").is_err() {
        std::env::set_var("ROCKET_ADDRESS", "0.0.0.0");
    }

    pretty_env_logger::init();

    sentry::init((
        "https://62fd0e02c5354905b4e286757f4beb16@sentry.insert.moe/4",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ))
}
