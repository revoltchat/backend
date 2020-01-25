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

fn gen_token(l: usize) -> String {
	rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(l)
		.collect::<String>()
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
		let access_token = gen_token(92);
		let code = gen_token(48);

		match col.insert_one(doc! {
			"_id": Ulid::new().to_string(),
			"email": info.email.clone(),
			"username": info.username.clone(),
			"password": hashed,
			"access_token": access_token,
			"email_verification": {
				"verified": false,
				"target": info.email.clone(),
				"expiry": UtcDatetime(Utc::now() + chrono::Duration::days(1)),
				"rate_limit": UtcDatetime(Utc::now() + chrono::Duration::minutes(1)),
				"code": code.clone(),
			}
		}, None) {
			Ok(_) => {
				let sent = email::send_verification_email(info.email.clone(), code);

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
			let ev = u.get_document("email_verification").expect("DOC[email_verification]");
			let expiry = ev.get_utc_datetime("expiry").expect("DOC[expiry]");

			if Utc::now() > *expiry {
				json!({
					"success": false,
					"error": "Token has expired!",
				})
			} else {
				let target = ev.get_str("target").expect("DOC[target]");
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
							"email": target.clone(),
						},
					},
					None,
				).expect("Failed to update user!");

				email::send_welcome_email(
					target.to_string(),
					u.get_str("username").expect("Failed to retrieve username.").to_string()
				);

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

#[derive(Serialize, Deserialize)]
pub struct Resend {
	email: String,
}

/// resend a verification email
/// (1) check if verification is pending for x email
/// (2) check for rate limit
/// (3) resend the email
#[post("/resend", data = "<info>")]
pub fn resend_email(info: Json<Resend>) -> JsonValue { 
	let col = database::get_db().collection("users");

	if let Some(u) =
		col.find_one(doc! { "email_verification.target": info.email.clone() }, None).expect("Failed user lookup") {
			let ev = u.get_document("email_verification").expect("DOC[email_verification]");
			let expiry = ev.get_utc_datetime("expiry").expect("DOC[expiry]");
			let rate_limit = ev.get_utc_datetime("rate_limit").expect("DOC[rate_limit]");

			if Utc::now() < *rate_limit {
				json!({
					"success": false,
					"error": "Hit rate limit! Please try again in a minute or so.",
				})
			} else {
				let mut new_expiry = UtcDatetime(Utc::now() + chrono::Duration::days(1));
				if info.email.clone() != u.get_str("email").expect("DOC[email]") {
					if Utc::now() > *expiry {
						return json!({
							"success": "false",
							"error": "For security reasons, please login and change your email again.",
						})
					}

					new_expiry = UtcDatetime(*expiry);
				}

				let code = gen_token(48);
				col.update_one(
					doc! { "_id": u.get_str("_id").expect("Failed to retrieve user id.") },
					doc! {
						"$set": {
							"email_verification.code": code.clone(),
							"email_verification.expiry": new_expiry,
							"email_verification.rate_limit": UtcDatetime(Utc::now() + chrono::Duration::minutes(1)),
						},
					},
					None,
				).expect("Failed to update user!");

				match email::send_verification_email(
					info.email.to_string(),
					code,
				) {
					true => json!({
						"success": true,
					}),
					false => json!({
						"success": false,
						"error": "Failed to send email! Likely an issue with the backend API.",
					})
				}
			}
		} else {
			json!({
				"success": false,
				"error": "Email not pending verification!",
			})
		}
}

#[derive(Serialize, Deserialize)]
pub struct Login {
	email: String,
	password: String,
}

/// login to a Revolt account
/// (1) find user by email
/// (2) verify password
/// (3) return access token
#[post("/login", data = "<info>")]
pub fn login(info: Json<Login>) -> JsonValue {
	let col = database::get_db().collection("users");

	if let Some(u) =
		col.find_one(doc! { "email": info.email.clone() }, None).expect("Failed user lookup") {
			match verify(info.password.clone(), u.get_str("password").expect("DOC[password]"))
				.expect("Failed to check hash of password.") {
					true => {
						let token =
							match u.get_str("access_token") {
								Ok(t) => t.to_string(),
								Err(_) => {
									let token = gen_token(92);
									col.update_one(
										doc! { "_id": u.get_str("_id").expect("DOC[id]") },
										doc! { "$set": { "access_token": token.clone() } },
										None
									).expect("Failed to update user object");
									token
								}
							};

						json!({
							"success": true,
							"access_token": token
						})
					},
					false => json!({
						"success": false,
						"error": "Invalid password."
					})
				}
		} else {
			json!({
				"success": false,
				"error": "Email is not registered.",
			})
		}
}
