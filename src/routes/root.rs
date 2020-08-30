use super::Response;
use crate::util::variables::{USE_EMAIL_VERIFICATION, USE_HCAPTCHA};

use mongodb::bson::doc;

/// root
#[get("/")]
pub fn root() -> Response {
    Response::Success(json!({
        "revolt": "0.2.9",
        "version": {
            "major": 0,
            "minor": 2,
            "patch": 9
        },
        "features": {
            "email_verification": USE_EMAIL_VERIFICATION.clone(),
            "captcha": USE_HCAPTCHA.clone(),
        }
    }))
}

#[options("/")]
pub fn root_preflight() -> Response {
    Response::Result(super::Status::Ok)
}

/// I'm a teapot.
#[delete("/")]
pub fn teapot() -> Response {
    Response::Teapot(json!({
        "teapot": true,
        "can_delete": false
    }))
}
