use crate::auth::User;
use crate::database;

use rocket_contrib::json::{ Json, JsonValue };
use serde::{ Serialize, Deserialize };
use mongodb::options::FindOptions;
use bson::{ bson, doc };

#[get("/@me")]
pub fn me(user: User) -> JsonValue {
	let User ( id, username, doc ) = user;

	json!({
		"id": id.to_string(),
		"username": username,
		"email": doc.get_str("email").expect("Missing email in user object!"),
		"verified": doc.get_document("email_verification").expect("DOC[email_verification]")
						.get_bool("verified").expect("DOC[verified]"),
		"created_timestamp": id.datetime().timestamp(),
	})
}

#[derive(Serialize, Deserialize)]
pub struct Query {
	username: String,
}

#[post("/lookup", data = "<query>")]
pub fn lookup(_user: User, query: Json<Query>) -> JsonValue {
	let col = database::get_db().collection("users");

	let users = col.find(
		doc! { "username": query.username.clone() },
		FindOptions::builder().limit(10).build()
	).expect("Failed user lookup");

	let mut results = Vec::new();
	for user in users {
		let u = user.expect("Failed to unwrap user.");
		results.push(
			json!({
				"id": u.get_str("_id").expect("DB[id]"),
				"username": u.get_str("username").expect("DB[username]")
			})
		);
	}

	json!(results)
}
