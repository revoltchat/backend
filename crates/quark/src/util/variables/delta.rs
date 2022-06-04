use std::env;

lazy_static! {
    // Application Settings
    pub static ref PUBLIC_URL: String =
        env::var("REVOLT_PUBLIC_URL").expect("Missing REVOLT_PUBLIC_URL environment variable.");
    pub static ref APP_URL: String =
        env::var("REVOLT_APP_URL").expect("Missing REVOLT_APP_URL environment variable.");
    pub static ref EXTERNAL_WS_URL: String =
        env::var("REVOLT_EXTERNAL_WS_URL").expect("Missing REVOLT_EXTERNAL_WS_URL environment variable.");

    pub static ref AUTUMN_URL: String =
        env::var("AUTUMN_PUBLIC_URL").unwrap_or_else(|_| "https://example.com".to_string());
    pub static ref JANUARY_URL: String =
        env::var("JANUARY_PUBLIC_URL").unwrap_or_else(|_| "https://example.com".to_string());
    pub static ref VOSO_URL: String =
        env::var("VOSO_PUBLIC_URL").unwrap_or_else(|_| "https://example.com".to_string());
    pub static ref VOSO_WS_HOST: String =
        env::var("VOSO_WS_HOST").unwrap_or_else(|_| "wss://example.com".to_string());
    pub static ref VOSO_MANAGE_TOKEN: String =
        env::var("VOSO_MANAGE_TOKEN").unwrap_or_else(|_| "0".to_string());

    pub static ref HCAPTCHA_KEY: String =
        env::var("REVOLT_HCAPTCHA_KEY").unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    pub static ref HCAPTCHA_SITEKEY: String =
        env::var("REVOLT_HCAPTCHA_SITEKEY").unwrap_or_else(|_| "10000000-ffff-ffff-ffff-000000000001".to_string());
    pub static ref VAPID_PRIVATE_KEY: String =
        env::var("REVOLT_VAPID_PRIVATE_KEY").expect("Missing REVOLT_VAPID_PRIVATE_KEY environment variable.");
    pub static ref VAPID_PUBLIC_KEY: String =
        env::var("REVOLT_VAPID_PUBLIC_KEY").expect("Missing REVOLT_VAPID_PUBLIC_KEY environment variable.");

    // Application Flags
    pub static ref INVITE_ONLY: bool = env::var("REVOLT_INVITE_ONLY").map_or(false, |v| v == "1");
    pub static ref USE_EMAIL: bool = env::var("REVOLT_USE_EMAIL_VERIFICATION").map_or(
        env::var("REVOLT_SMTP_HOST").is_ok()
            && env::var("REVOLT_SMTP_USERNAME").is_ok()
            && env::var("REVOLT_SMTP_PASSWORD").is_ok()
            && env::var("REVOLT_SMTP_FROM").is_ok(),
        |v| v == *"1"
    );
    pub static ref USE_HCAPTCHA: bool = env::var("REVOLT_HCAPTCHA_KEY").is_ok();
    pub static ref USE_AUTUMN: bool = env::var("AUTUMN_PUBLIC_URL").is_ok();
    pub static ref USE_JANUARY: bool = env::var("JANUARY_PUBLIC_URL").is_ok();
    pub static ref USE_VOSO: bool = env::var("VOSO_PUBLIC_URL").is_ok() && env::var("VOSO_MANAGE_TOKEN").is_ok();

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
    pub static ref MAX_BOT_COUNT: usize =
        env::var("REVOLT_MAX_BOT_COUNT").unwrap_or_else(|_| "5".to_string()).parse().unwrap();
    pub static ref MAX_EMBED_COUNT: usize =
        env::var("REVOLT_MAX_EMBED_COUNT").unwrap_or_else(|_| "5".to_string()).parse().unwrap();
    pub static ref MAX_SERVER_COUNT: usize =
        env::var("REVOLT_MAX_SERVER_COUNT").unwrap_or_else(|_| "100".to_string()).parse().unwrap();
    pub static ref EARLY_ADOPTER_BADGE: i64 =
        env::var("REVOLT_EARLY_ADOPTER_BADGE").unwrap_or_else(|_| "0".to_string()).parse().unwrap();
}

pub fn preflight_checks() {
    format!("url = {}", *APP_URL);
    format!("public = {}", *PUBLIC_URL);
    format!("external = {}", *EXTERNAL_WS_URL);

    format!("privkey = {}", *VAPID_PRIVATE_KEY);
    format!("pubkey = {}", *VAPID_PUBLIC_KEY);

    if !(*USE_EMAIL) {
        #[cfg(not(debug_assertions))]
        if !env::var("REVOLT_UNSAFE_NO_EMAIL").map_or(false, |v| v == *"1") {
            panic!("Running in production without email is not recommended, set REVOLT_UNSAFE_NO_EMAIL=1 to override.");
        }

        #[cfg(debug_assertions)]
        warn!("No SMTP settings specified! Remember to configure email.");
    }

    if !(*USE_HCAPTCHA) {
        #[cfg(not(debug_assertions))]
        if !env::var("REVOLT_UNSAFE_NO_CAPTCHA").map_or(false, |v| v == *"1") {
            panic!("Running in production without CAPTCHA is not recommended, set REVOLT_UNSAFE_NO_CAPTCHA=1 to override.");
        }

        #[cfg(debug_assertions)]
        warn!("No Captcha key specified! Remember to add hCaptcha key.");
    }
}
