/// Configure logging and common Rust variables
pub fn setup_logging(release: &'static str) -> Option<sentry::ClientInitGuard> {
    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    if std::env::var("ROCKET_ADDRESS").is_err() {
        std::env::set_var("ROCKET_ADDRESS", "0.0.0.0");
    }

    pretty_env_logger::init();
    info!("Starting {release}");

    if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        Some(sentry::init((
            dsn,
            sentry::ClientOptions {
                release: Some(release.into()),
                ..Default::default()
            },
        )))
    } else {
        None
    }
}

#[macro_export]
macro_rules! configure {
    () => {
        let _sentry = revolt_quark::util::log::setup_logging(concat!(
            env!("CARGO_PKG_NAME"),
            "@",
            env!("CARGO_PKG_VERSION")
        ));
    };
}
