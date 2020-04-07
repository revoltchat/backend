use super::Response;
use crate::database;
use crate::email;

use bcrypt::{hash, verify};
use bson::{bson, doc, from_bson, Bson::UtcDatetime};
use chrono::prelude::*;
use database::user::User;
use rand::{distributions::Alphanumeric, Rng};
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::validate_email;

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
pub fn create(info: Json<Create>) -> Response {
    let col = database::get_collection("users");

    if info.username.len() < 2 || info.username.len() > 32 {
        return Response::NotAcceptable(
            json!({ "error": "Username needs to be at least 2 chars and less than 32 chars." }),
        );
    }

    if info.password.len() < 8 || info.password.len() > 72 {
        return Response::NotAcceptable(
            json!({ "error": "Password needs to be at least 8 chars and at most 72." }),
        );
    }

    if !validate_email(info.email.clone()) {
        return Response::UnprocessableEntity(json!({ "error": "Invalid email." }));
    }

    if let Some(_) = col
        .find_one(doc! { "email": info.email.clone() }, None)
        .expect("Failed user lookup")
    {
        return Response::Conflict(json!({ "error": "Email already in use!" }));
    }

    if let Ok(hashed) = hash(info.password.clone(), 10) {
        let access_token = gen_token(92);
        let code = gen_token(48);

        match col.insert_one(
            doc! {
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
            },
            None,
        ) {
            Ok(_) => {
                let sent = email::send_verification_email(info.email.clone(), code);

                Response::Success(json!({
                    "success": true,
                    "email_sent": sent,
                }))
            }
            Err(_) => {
                Response::InternalServerError(json!({ "error": "Failed to create account." }))
            }
        }
    } else {
        Response::InternalServerError(json!({ "error": "Failed to hash." }))
    }
}

/// verify an email for a Revolt account
/// (1) check if code is valid
/// (2) check if it expired yet
/// (3) set account as verified
#[get("/verify/<code>")]
pub fn verify_email(code: String) -> Response {
    let col = database::get_collection("users");

    if let Some(u) = col
        .find_one(doc! { "email_verification.code": code.clone() }, None)
        .expect("Failed user lookup")
    {
        let user: User = from_bson(bson::Bson::Document(u)).expect("Failed to unwrap user.");
        let ev = user.email_verification;

        if Utc::now() > *ev.expiry.unwrap() {
            Response::Gone(json!({
                "success": false,
                "error": "Token has expired!",
            }))
        } else {
            let target = ev.target.unwrap();
            col.update_one(
                doc! { "_id": user.id },
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
            )
            .expect("Failed to update user!");

            email::send_welcome_email(target.to_string(), user.username);

            Response::Redirect(
                super::Redirect::to("https://example.com"), // ! FIXME; redirect to landing page
            )
        }
    } else {
        Response::BadRequest(json!({ "error": "Invalid code." }))
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
pub fn resend_email(info: Json<Resend>) -> Response {
    let col = database::get_collection("users");

    if let Some(u) = col
        .find_one(
            doc! { "email_verification.target": info.email.clone() },
            None,
        )
        .expect("Failed user lookup")
    {
        let user: User = from_bson(bson::Bson::Document(u)).expect("Failed to unwrap user.");
        let ev = user.email_verification;

        let expiry = ev.expiry.unwrap();
        let rate_limit = ev.rate_limit.unwrap();

        if Utc::now() < *rate_limit {
            Response::TooManyRequests(
                json!({ "error": "You are being rate limited, please try again in a while." }),
            )
        } else {
            let mut new_expiry = UtcDatetime(Utc::now() + chrono::Duration::days(1));
            if info.email.clone() != user.email {
                if Utc::now() > *expiry {
                    return Response::Gone(
                        json!({ "error": "To help protect your account, please login and change your email again. The original request was made over one day ago." }),
                    );
                }

                new_expiry = UtcDatetime(*expiry);
            }

            let code = gen_token(48);
            col.update_one(
					doc! { "_id": user.id },
					doc! {
						"$set": {
							"email_verification.code": code.clone(),
							"email_verification.expiry": new_expiry,
							"email_verification.rate_limit": UtcDatetime(Utc::now() + chrono::Duration::minutes(1)),
						},
					},
					None,
				).expect("Failed to update user!");

            match email::send_verification_email(info.email.to_string(), code) {
                true => Response::Result(super::Status::Ok),
                false => Response::InternalServerError(
                    json!({ "success": false, "error": "Failed to send email! Likely an issue with the backend API." }),
                ),
            }
        }
    } else {
        Response::NotFound(
            json!({ "success": false, "error": "Email not found or pending verification!" }),
        )
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
pub fn login(info: Json<Login>) -> Response {
    let col = database::get_collection("users");

    if let Some(u) = col
        .find_one(doc! { "email": info.email.clone() }, None)
        .expect("Failed user lookup")
    {
        let user: User = from_bson(bson::Bson::Document(u)).expect("Failed to unwrap user.");

        match verify(info.password.clone(), &user.password)
            .expect("Failed to check hash of password.")
        {
            true => {
                let token = match user.access_token {
                    Some(t) => t.to_string(),
                    None => {
                        let token = gen_token(92);
                        col.update_one(
                            doc! { "_id": &user.id },
                            doc! { "$set": { "access_token": token.clone() } },
                            None,
                        )
                        .expect("Failed to update user object");
                        token
                    }
                };

                Response::Success(json!({ "access_token": token, "id": user.id }))
            }
            false => Response::Unauthorized(json!({ "error": "Invalid password." })),
        }
    } else {
        Response::NotFound(json!({ "error": "Email is not registered." }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    token: String,
}

/// login to a Revolt account via token
#[post("/token", data = "<info>")]
pub fn token(info: Json<Token>) -> Response {
    let col = database::get_collection("users");

    if let Some(u) = col
        .find_one(doc! { "access_token": info.token.clone() }, None)
        .expect("Failed user lookup")
    {
        Response::Success(json!({
            "id": u.get_str("_id").unwrap(),
        }))
    } else {
        Response::Unauthorized(json!({
            "error": "Invalid token!",
        }))
    }
}
