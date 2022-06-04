use revolt_quark::variables::delta::{
    APP_URL, AUTUMN_URL, EXTERNAL_WS_URL, HCAPTCHA_SITEKEY, INVITE_ONLY, JANUARY_URL, USE_AUTUMN,
    USE_EMAIL, USE_HCAPTCHA, USE_JANUARY, USE_VOSO, VAPID_PUBLIC_KEY, VOSO_URL, VOSO_WS_HOST,
};
use revolt_quark::Result;

use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Serialize;

/// # hCaptcha Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct CaptchaFeature {
    /// Whether captcha is enabled
    pub enabled: bool,
    /// Client key used for solving captcha
    pub key: String,
}

/// # Generic Service Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct Feature {
    /// Whether the service is enabled
    pub enabled: bool,
    /// URL pointing to the service
    pub url: String,
}

/// # Voice Server Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct VoiceFeature {
    /// Whether voice is enabled
    pub enabled: bool,
    /// URL pointing to the voice API
    pub url: String,
    /// URL pointing to the voice WebSocket server
    pub ws: String,
}

/// # Feature Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct RevoltFeatures {
    /// hCaptcha configuration
    pub captcha: CaptchaFeature,
    /// Whether email verification is enabled
    pub email: bool,
    /// Whether this server is invite only
    pub invite_only: bool,
    /// File server service configuration
    pub autumn: Feature,
    /// Proxy service configuration
    pub january: Feature,
    /// Voice server configuration
    pub voso: VoiceFeature,
}

/// # Server Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct RevoltConfig {
    /// Revolt API Version
    pub revolt: String,
    /// Features enabled on this Revolt node
    pub features: RevoltFeatures,
    /// WebSocket URL
    pub ws: String,
    /// URL pointing to the client serving this node
    pub app: String,
    /// Web Push VAPID public key
    pub vapid: String,
}

/// # Query Node
///
/// Fetch the server configuration for this Revolt instance.
#[openapi(tag = "Core")]
#[get("/")]
pub async fn root() -> Result<Json<RevoltConfig>> {
    Ok(Json(RevoltConfig {
        revolt: env!("CARGO_PKG_VERSION").to_string(),
        features: RevoltFeatures {
            captcha: CaptchaFeature {
                enabled: *USE_HCAPTCHA,
                key: HCAPTCHA_SITEKEY.to_string(),
            },
            email: *USE_EMAIL,
            invite_only: *INVITE_ONLY,
            autumn: Feature {
                enabled: *USE_AUTUMN,
                url: AUTUMN_URL.to_string(),
            },
            january: Feature {
                enabled: *USE_JANUARY,
                url: JANUARY_URL.to_string(),
            },
            voso: VoiceFeature {
                enabled: *USE_VOSO,
                url: VOSO_URL.to_string(),
                ws: VOSO_WS_HOST.to_string(),
            },
        },
        ws: EXTERNAL_WS_URL.to_string(),
        app: APP_URL.to_string(),
        vapid: VAPID_PUBLIC_KEY.to_string(),
    }))
}

/// Example endpoint.
#[openapi(skip)]
#[get("/ping")]
pub async fn ping(/*_limitguard: Ratelimiter*/) -> Status {
    Status::Ok
}
