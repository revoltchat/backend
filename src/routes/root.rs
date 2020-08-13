use super::Response;

use mongodb::bson::doc;
use std::env;

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
            "captcha": env::var("HCAPTCHA_KEY").is_ok()
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
