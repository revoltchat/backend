use crate::util::variables::{DISABLE_REGISTRATION, HCAPTCHA_SITEKEY, USE_EMAIL, USE_HCAPTCHA};

use rocket_contrib::json::JsonValue;
use mongodb::bson::doc;

/// root
#[get("/")]
pub async fn root() -> JsonValue {
    json!({
        "revolt": "0.3.0-alpha",
        "features": {
            "registration": !*DISABLE_REGISTRATION,
            "captcha": {
                "enabled": *USE_HCAPTCHA,
                "key": HCAPTCHA_SITEKEY.to_string()
            },
            "email": *USE_EMAIL,
        }
    })
}

/// I'm a teapot.
#[delete("/")]
pub async fn teapot() -> JsonValue {
    json!({
        "teapot": true,
        "can_delete": false
    })
}
