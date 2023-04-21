use std::env;

use once_cell::sync::Lazy;

pub static REGION_ID: Lazy<u16> = Lazy::new(|| {
    env::var("REGION_ID")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .unwrap()
});

pub static REGION_KEY: Lazy<String> = Lazy::new(|| format!("region{}", &*REGION_ID));
