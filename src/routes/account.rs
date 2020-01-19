use crate::auth::User;
use crate::database;

use serde::{Serialize, Deserialize};
use rocket_contrib::json::{ Json, JsonValue };
use bson::{ bson, doc };

#[get("/")]
pub fn root(user: User) -> String {
	let User ( id, username, _doc ) = user;

	format!("hello, {}! [id: {}]", username, id)
}

#[derive(Serialize, Deserialize)]
pub struct Create {
	username: String,
	password: String,
	email: String,
}

#[post("/create", data = "<info>")]
pub fn create(info: Json<Create>) -> JsonValue {
	let col = database::get_db().collection("users");

	if let Some(_) =
			col.find_one(Some(
				doc! { "email": info.email.clone() }
			), None).unwrap() {

		return json!({
			"success": false,
			"error": "Email already in use!"
		})
	}

	json!({
		"success": true
	})
}
