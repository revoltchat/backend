use std::str::FromStr;

pub use rocket_cors::catch_all_options_routes;
use rocket_cors::{AllowedOrigins, Cors};

pub fn new() -> Cors {
    rocket_cors::CorsOptions {
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
    .expect("Failed to create CORS.")
}
