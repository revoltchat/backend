use super::Response;

use bson::doc;

/// root
#[get("/")]
pub fn root() -> Response {
    Response::Success(json!({
        "revolt": "0.2.2"
    }))
}

/// I'm a teapot.
#[delete("/")]
pub fn teapot() -> Response {
    Response::Teapot(json!({
        "teapot": true,
        "can_delete": false
    }))
}
