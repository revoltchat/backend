use super::Response;

use mongodb::bson::doc;

/// root
#[get("/")]
pub fn root() -> Response {
    Response::Success(json!({
        "revolt": "0.2.5"
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
