use super::Response;
use crate::util::variables::{USE_EMAIL, DISABLE_REGISTRATION, USE_HCAPTCHA};

use mongodb::bson::doc;

/// root
#[get("/")]
pub fn root() -> Response {
    Response::Success(json!({
        "revolt": "0.2.10",
        "version": {
            "major": 0,
            "minor": 2,
            "patch": 10
        },
        "features": {
            "registration": !*DISABLE_REGISTRATION,
            "captcha": *USE_HCAPTCHA,
            "email": *USE_EMAIL,
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
