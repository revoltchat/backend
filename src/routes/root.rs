use crate::util::variables::{
    APP_URL, AUTUMN_URL, DISABLE_REGISTRATION, EXTERNAL_WS_URL, HCAPTCHA_SITEKEY, INVITE_ONLY,
    JANUARY_URL, USE_AUTUMN, USE_EMAIL, USE_HCAPTCHA, USE_JANUARY, USE_VOSO, VAPID_PUBLIC_KEY,
    VOSO_URL, VOSO_WS_HOST,
};

use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[get("/")]
pub async fn root() -> JsonValue {
    json!({
        "revolt": crate::version::VERSION,
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
            "january": {
                "enabled": *USE_JANUARY,
                "url": *JANUARY_URL
            },
            "voso": {
                "enabled": *USE_VOSO,
                "url": *VOSO_URL,
                "ws": *VOSO_WS_HOST
            }
        },
        "ws": *EXTERNAL_WS_URL,
        "app": *APP_URL,
        "vapid": *VAPID_PUBLIC_KEY
    })
}
