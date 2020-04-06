use rocket_contrib::json::{ JsonValue };
use bson::{ doc };

/// root
#[get("/")]
pub fn root() -> JsonValue {
	json!({
		"revolt": "0.0.1"
	})
}
