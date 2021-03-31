use crate::util::variables::{
    AUTUMN_URL, DISABLE_REGISTRATION, EXTERNAL_WS_URL, HCAPTCHA_SITEKEY, INVITE_ONLY, USE_AUTUMN,
    USE_EMAIL, USE_HCAPTCHA, VAPID_PUBLIC_KEY, USE_VOSO, VOSO_URL
};

use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[get("/")]
pub async fn root() -> JsonValue {
    json!({
        "revolt": "0.4.0-alpha.0",
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
            },
            "voso": {
                "enabled": *USE_VOSO,
                "url": *VOSO_URL
            }
        },
        "ws": *EXTERNAL_WS_URL,
        "vapid": *VAPID_PUBLIC_KEY
    })
}
