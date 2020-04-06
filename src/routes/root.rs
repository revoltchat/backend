use bson::doc;
use rocket_contrib::json::JsonValue;

/// root
#[get("/")]
pub fn root() -> JsonValue {
    json!({
        "revolt": "0.0.1"
    })
}
