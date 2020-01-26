use crate::auth::User;
use crate::database;

use rocket_contrib::json::{ Json, JsonValue };
use serde::{ Serialize, Deserialize };
use mongodb::options::FindOptions;
use bson::{ bson, doc };

/// retrieve your user information
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

/// retrieve another user's information
#[get("/<id>")]
pub fn user(user: User, id: String) -> JsonValue {
	json!([])
}

#[derive(Serialize, Deserialize)]
pub struct Query {
	username: String,
}

/// lookup a user on Revolt
/// currently only supports exact username searches
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

/// retrieve all of your DMs
#[get("/@me/dms")]
pub fn dms(user: User) -> JsonValue {
	json!([])
}

/// open a DM with a user
#[get("/<id>/dm")]
pub fn dm(user: User, id: String) -> JsonValue {
	json!([])
}

/// retrieve all of your friends
#[get("/@me/friend")]
pub fn get_friends(user: User) -> JsonValue {
	json!([])
}

/// retrieve friend status with user
#[get("/<id>/friend")]
pub fn get_friend(user: User, id: String) -> JsonValue {
	json!([])
}

/// create or accept a friend request
#[put("/<id>/friend")]
pub fn add_friend(user: User, id: String) -> JsonValue {
	json!([])
}

/// remove a friend or deny a request
#[delete("/<id>/friend")]
pub fn remove_friend(user: User, id: String) -> JsonValue {
	json!([])
}
