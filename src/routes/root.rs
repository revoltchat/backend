use super::Response;

use bson::doc;

/// root
#[get("/")]
pub fn root() -> Response {
    Response::Success(json!({
        "revolt": "0.1.0"
    }))
}
