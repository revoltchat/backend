use std::env;

#[cfg(debug_assertions)]
use log::warn;

lazy_static! {
    // Application Settings
    pub static ref MONGO_URI: String =
        env::var("REVOLT_MONGO_URI").expect("Missing REVOLT_MONGO_URI environment variable.");
    pub static ref PUBLIC_URL: String =
        env::var("REVOLT_PUBLIC_URL").expect("Missing REVOLT_PUBLIC_URL environment variable.");
    pub static ref APP_URL: String =
        env::var("REVOLT_APP_URL").unwrap_or_else(|_| "https://app.revolt.chat".to_string());
    pub static ref EXTERNAL_WS_URL: String =
        env::var("REVOLT_EXTERNAL_WS_URL").expect("Missing REVOLT_EXTERNAL_WS_URL environment variable.");
    pub static ref HCAPTCHA_KEY: String =
        env::var("REVOLT_HCAPTCHA_KEY").unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    pub static ref HCAPTCHA_SITEKEY: String =
        env::var("REVOLT_HCAPTCHA_SITEKEY").unwrap_or_else(|_| "10000000-ffff-ffff-ffff-000000000001".to_string());
    pub static ref WS_HOST: String =
        env::var("REVOLT_WS_HOST").unwrap_or_else(|_| "0.0.0.0:9000".to_string());

    // Application Flags
    pub static ref DISABLE_REGISTRATION: bool = env::var("REVOLT_DISABLE_REGISTRATION").map_or(false, |v| v == "1");
    pub static ref USE_EMAIL: bool = env::var("REVOLT_USE_EMAIL_VERIFICATION").map_or(
        env::var("REVOLT_SMTP_HOST").is_ok()
            && env::var("REVOLT_SMTP_USERNAME").is_ok()
            && env::var("REVOLT_SMTP_PASSWORD").is_ok()
            && env::var("REVOLT_SMTP_FROM").is_ok(),
        |v| v == *"1"
    );
    pub static ref USE_HCAPTCHA: bool = env::var("REVOLT_HCAPTCHA_KEY").is_ok();
    pub static ref USE_PROMETHEUS: bool = env::var("REVOLT_ENABLE_PROMETHEUS").map_or(false, |v| v == "1");

    // SMTP Settings
    pub static ref SMTP_HOST: String =
        env::var("REVOLT_SMTP_HOST").unwrap_or_else(|_| "".to_string());
    pub static ref SMTP_USERNAME: String =
        env::var("REVOLT_SMTP_USERNAME").unwrap_or_else(|_| "".to_string());
    pub static ref SMTP_PASSWORD: String =
        env::var("REVOLT_SMTP_PASSWORD").unwrap_or_else(|_| "".to_string());
    pub static ref SMTP_FROM: String = env::var("REVOLT_SMTP_FROM").unwrap_or_else(|_| "".to_string());

    // Application Logic Settings
    pub static ref MAX_GROUP_SIZE: usize =
        env::var("REVOLT_MAX_GROUP_SIZE").unwrap_or_else(|_| "50".to_string()).parse().unwrap();
}

pub fn preflight_checks() {
    if *USE_EMAIL == false {
        #[cfg(not(debug_assertions))]
        if !env::var("REVOLT_UNSAFE_NO_EMAIL").map_or(false, |v| v == *"1") {
            panic!("Running in production without email is not recommended, set REVOLT_UNSAFE_NO_EMAIL=1 to override.");
        }

        #[cfg(debug_assertions)]
        warn!("No SMTP settings specified! Remember to configure email.");
    }

    if *USE_HCAPTCHA == false {
        #[cfg(not(debug_assertions))]
        if !env::var("REVOLT_UNSAFE_NO_CAPTCHA").map_or(false, |v| v == *"1") {
            panic!("Running in production without CAPTCHA is not recommended, set REVOLT_UNSAFE_NO_CAPTCHA=1 to override.");
        }

        #[cfg(debug_assertions)]
        warn!("No Captcha key specified! Remember to add hCaptcha key.");
    }
}
