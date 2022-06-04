/// Configure logging and common Rust variables
pub fn setup_logging(release: &'static str) -> sentry::ClientInitGuard {
    dotenv::dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    if std::env::var("ROCKET_ADDRESS").is_err() {
        std::env::set_var("ROCKET_ADDRESS", "0.0.0.0");
    }

    pretty_env_logger::init();
    info!("Starting {release}");

    sentry::init((
        "https://62fd0e02c5354905b4e286757f4beb16@sentry.insert.moe/4",
        sentry::ClientOptions {
            release: Some(release.into()),
            ..Default::default()
        },
    ))
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
