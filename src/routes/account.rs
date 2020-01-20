use crate::auth::User;
use crate::database;

use rand::{ Rng, distributions::Alphanumeric };
use rocket_contrib::json::{ Json, JsonValue };
use bson::{ bson, doc, Bson::UtcDatetime };
use serde::{ Serialize, Deserialize };
use validator::validate_email;
use bcrypt::{ hash, verify };
use chrono::prelude::*;
use ulid::Ulid;

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

/// create a new Revolt account
/// (1) validate input
/// 	[username] 2 to 32 characters
/// 	[password] 8 to 72 characters
/// 	[email]    validate against RFC
/// (2) check email existence
/// (3) add user and send email verification
#[post("/create", data = "<info>")]
pub fn create(info: Json<Create>) -> JsonValue {
	let col = database::get_db().collection("users");

	if info.username.len() < 2 || info.username.len() > 32 {
		return json!({
			"success": false,
			"error": "Username requirements not met! Must be between 2 and 32 characters.",
		})
	}

	if info.password.len() < 8 || info.password.len() > 72 {
		return json!({
			"success": false,
			"error": "Password requirements not met! Must be between 8 and 72 characters.",
		})
	}

	if !validate_email(info.email.clone()) {
		return json!({
			"success": false,
			"error": "Invalid email provided!",
		})
	}

	if let Some(_) =
			col.find_one(Some(
				doc! { "email": info.email.clone() }
			), None).unwrap() {

		return json!({
			"success": false,
			"error": "Email already in use!",
		})
	}

	if let Ok(hashed) = hash(info.password.clone(), 10) {
		let code = rand::thread_rng()
			.sample_iter(&Alphanumeric)
			.take(48)
			.collect::<String>();

		match col.insert_one(doc! {
			"_id": Ulid::new().to_string(),
			"email": info.email.clone(),
			"username": info.username.clone(),
			"password": hashed,
			"email_verification": {
				"verified": false,
				"target": info.email.clone(),
				"expiry": UtcDatetime(Utc::now() + chrono::Duration::days(1)),
				"code": code,
			}
		}, None) {
			Ok(_) => json!({
				"success": true,
			}),
			Err(_) => json!({
				"success": false,
				"error": "Failed to create account!",
			})
		}
	} else {
		json!({
			"success": false,
			"error": "Failed to hash password!",
		})
	}
}
