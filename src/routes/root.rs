use crate::util::variables::{
    DISABLE_REGISTRATION, EXTERNAL_WS_URL, HCAPTCHA_SITEKEY, INVITE_ONLY, USE_EMAIL, USE_HCAPTCHA, USE_AUTUMN, AUTUMN_URL
};

use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[get("/")]
pub async fn root() -> JsonValue {
    json!({
        "revolt": "0.3.3-alpha.5",
        "features": {
            "registration": !*DISABLE_REGISTRATION,
            "captcha": {
                "enabled": *USE_HCAPTCHA,
                "key": HCAPTCHA_SITEKEY.to_string()
            },
            "email": *USE_EMAIL,
            "invite_only": *INVITE_ONLY,
            "autumn": {
                "enabled": *USE_AUTUMN,
                "url": *AUTUMN_URL
            }
        },
        "ws": *EXTERNAL_WS_URL,
    })
}
