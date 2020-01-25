use crate::auth::User;
use crate::database;
use crate::email;

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

	if let Some(_) = col.find_one(doc! { "email": info.email.clone() }, None).expect("Failed user lookup") {
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
				"expiry": UtcDatetime(Utc::now() + chrono::Duration::seconds(1)),
				"rate_limit": UtcDatetime(Utc::now() + chrono::Duration::minutes(1)),
				"code": code.clone(),
			}
		}, None) {
			Ok(_) => {
				let url = format!("http://192.168.0.10:5500/api/account/verify/{}", code);
				let sent =
					match email::send_email(
						info.email.clone(),
						"Verify your email!".to_string(),
						format!("Verify your email here: {}", url).to_string(),
						format!("<a href=\"{}\">Click to verify your email!</a>", url).to_string()
					) {
						Ok(_) => true,
						Err(_) => false,
					};

				json!({
					"success": true,
					"email_sent": sent,
				})
			},
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

/// verify an email for a Revolt account
/// (1) check if code is valid
/// (2) check if it expired yet
/// (3) set account as verified
#[get("/verify/<code>")]
pub fn verify_email(code: String) -> JsonValue {
	let col = database::get_db().collection("users");

	if let Some(u) =
		col.find_one(doc! { "email_verification.code": code.clone() }, None).expect("Failed user lookup") {
			let ev = u.get_document("email_verification").expect("Missing email_verification on user object!");
			let expiry = ev.get_utc_datetime("expiry").expect("Missing expiry date on email_verification!");

			if Utc::now() > *expiry {
				json!({
					"success": false,
					"error": "Token has expired!",
				})
			} else {
				let target = ev.get_str("target").expect("Missing target email on email_verification!");
				col.update_one(
					doc! { "_id": u.get_str("_id").expect("Failed to retrieve user id.") },
					doc! {
						"$unset": {
							"email_verification.code": "",
							"email_verification.expiry": "",
							"email_verification.target": "",
							"email_verification.rate_limit": "",
						},
						"$set": {
							"email_verification.verified": true,
							"email": target,
						},
					},
					None,
				).expect("Failed to update user!");

				json!({
					"success": true
				})
			}
		} else {
			json!({
				"success": false,
				"error": "Invalid code!",
			})
		}
}
