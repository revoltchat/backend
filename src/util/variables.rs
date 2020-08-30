use std::env;

lazy_static! {
    pub static ref MONGO_URI: String =
        env::var("REVOLT_MONGO_URI").expect("Missing REVOLT_MONGO_URI environment variable.");
    pub static ref PUBLIC_URL: String =
        env::var("REVOLT_PUBLIC_URL").expect("Missing REVOLT_PUBLIC_URL environment variable.");
    pub static ref USE_EMAIL_VERIFICATION: bool = env::var("REVOLT_USE_EMAIL_VERIFICATION").map_or(
        env::var("REVOLT_SMTP_HOST").is_ok()
            && env::var("REVOLT_SMTP_USERNAME").is_ok()
            && env::var("REVOLT_SMTP_PASSWORD").is_ok()
            && env::var("REVOLT_SMTP_FROM").is_ok(),
        |v| v == *"1"
    );
    pub static ref USE_HCAPTCHA: bool = env::var("REVOLT_HCAPTCHA_KEY").is_ok();
    pub static ref SMTP_HOST: String =
        env::var("REVOLT_SMTP_HOST").unwrap_or_else(|_| "".to_string());
    pub static ref SMTP_USERNAME: String =
        env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string());
    pub static ref SMTP_PASSWORD: String =
        env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".to_string());
    pub static ref SMTP_FROM: String = env::var("SMTP_FROM").unwrap_or_else(|_| "".to_string());
    pub static ref HCAPTCHA_KEY: String =
        env::var("REVOLT_HCAPTCHA_KEY").unwrap_or_else(|_| "".to_string());
    pub static ref WS_HOST: String =
        env::var("REVOLT_WS_HOST").unwrap_or_else(|_| "0.0.0.0:9999".to_string());
}
