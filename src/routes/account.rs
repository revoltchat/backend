use super::Response;
use crate::database;
use crate::util::{captcha, email, gen_token};
use crate::util::variables::{DISABLE_REGISTRATION, USE_EMAIL};

use bcrypt::{hash, verify};
use chrono::prelude::*;
use database::user::User;
use log::error;
use mongodb::bson::{doc, from_bson, Bson};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::validate_email;

#[derive(Serialize, Deserialize)]
pub struct Create {
    username: String,
    password: String,
    email: String,
    captcha: Option<String>,
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
    if let Err(error) = captcha::verify(&info.captcha) {
        return Response::BadRequest(json!({ "error": error }));
    }

    if *DISABLE_REGISTRATION {
        return Response::BadRequest(json!({ "error": "Registration disabled." }));
    }

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

    if let Some(_) = col
        .find_one(doc! { "username": info.username.clone() }, None)
        .expect("Failed user lookup")
    {
        return Response::Conflict(json!({ "error": "Username already in use!" }));
    }

    if let Ok(hashed) = hash(info.password.clone(), 10) {
        let access_token = gen_token(92);
        let code = gen_token(48);

        let email_verification = match *USE_EMAIL {
            true => doc! {
                "verified": false,
                "target": info.email.clone(),
                "expiry": Bson::DateTime(Utc::now() + chrono::Duration::days(1)),
                "rate_limit": Bson::DateTime(Utc::now() + chrono::Duration::minutes(1)),
                "code": code.clone(),
            },
            false => doc! {
                "verified": true
            }
        };

        let id = Ulid::new().to_string();
        match col.insert_one(
            doc! {
                "_id": &id,
                "email": info.email.clone(),
                "username": info.username.clone(),
                "display_name": info.username.clone(),
                "password": hashed,
                "access_token": &access_token,
                "email_verification": email_verification
            },
            None,
        ) {
            Ok(_) => {
                if *USE_EMAIL {
                    let sent = email::send_verification_email(info.email.clone(), code);

                    Response::Success(json!({
                        "email_sent": sent,
                    }))
                } else {
                    Response::Success(json!({
                        "id": id,
                        "access_token": access_token
                    }))
                }
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
        let user: User = from_bson(Bson::Document(u)).expect("Failed to unwrap user.");
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

            if *USE_EMAIL {
                if let Err(err) = email::send_welcome_email(target.to_string(), user.username) {
                    error!("Failed to send welcome email! {}", err);
                }
            }

            Response::Redirect(super::Redirect::to("https://app.revolt.chat"))
        }
    } else {
        Response::BadRequest(json!({ "error": "Invalid code." }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Resend {
    email: String,
    captcha: Option<String>,
}

/// resend a verification email
/// (1) check if verification is pending for x email
/// (2) check for rate limit
/// (3) resend the email
#[post("/resend", data = "<info>")]
pub fn resend_email(info: Json<Resend>) -> Response {
    if let Err(error) = captcha::verify(&info.captcha) {
        return Response::BadRequest(json!({ "error": error }));
    }

    let col = database::get_collection("users");

    if let Some(u) = col
        .find_one(
            doc! { "email_verification.target": info.email.clone() },
            None,
        )
        .expect("Failed user lookup.")
    {
        let user: User = from_bson(Bson::Document(u)).expect("Failed to unwrap user.");
        let ev = user.email_verification;

        let expiry = ev.expiry.unwrap();
        let rate_limit = ev.rate_limit.unwrap();

        if Utc::now() < *rate_limit {
            Response::TooManyRequests(
                json!({ "error": "You are being rate limited, please try again in a while." }),
            )
        } else {
            let mut new_expiry = Bson::DateTime(Utc::now() + chrono::Duration::days(1));
            if info.email.clone() != user.email {
                if Utc::now() > *expiry {
                    return Response::Gone(
                        json!({ "error": "To help protect your account, please login and change your email again. The original request was made over one day ago." }),
                    );
                }

                new_expiry = Bson::DateTime(*expiry);
            }

            let code = gen_token(48);
            col.update_one(
					doc! { "_id": user.id },
					doc! {
						"$set": {
							"email_verification.code": code.clone(),
							"email_verification.expiry": new_expiry,
							"email_verification.rate_limit": Bson::DateTime(Utc::now() + chrono::Duration::minutes(1)),
						},
					},
					None,
				).expect("Failed to update user!");

            if let Err(err) = email::send_verification_email(info.email.clone(), code) {
                return Response::InternalServerError(json!({ "error": err }));
            }

            Response::Result(super::Status::Ok)
        }
    } else {
        Response::NotFound(json!({ "error": "Email not found or pending verification!" }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    email: String,
    password: String,
    captcha: Option<String>,
}

/// login to a Revolt account
/// (1) find user by email
/// (2) verify password
/// (3) return access token
#[post("/login", data = "<info>")]
pub fn login(info: Json<Login>) -> Response {
    if let Err(error) = captcha::verify(&info.captcha) {
        return Response::BadRequest(json!({ "error": error }));
    }

    let col = database::get_collection("users");

    if let Some(u) = col
        .find_one(doc! { "email": info.email.clone() }, None)
        .expect("Failed user lookup")
    {
        let user: User = from_bson(Bson::Document(u)).expect("Failed to unwrap user.");

        match verify(info.password.clone(), &user.password)
            .expect("Failed to check hash of password.")
        {
            true => {
                let token = match user.access_token {
                    Some(t) => t.to_string(),
                    None => {
                        let token = gen_token(92);
                        if col
                            .update_one(
                                doc! { "_id": &user.id },
                                doc! { "$set": { "access_token": token.clone() } },
                                None,
                            )
                            .is_err()
                        {
                            return Response::InternalServerError(
                                json!({ "error": "Failed database operation." }),
                            );
                        }

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

    if let Ok(result) = col.find_one(doc! { "access_token": info.token.clone() }, None) {
        if let Some(user) = result {
            Response::Success(json!({
                "id": user.get_str("_id").unwrap(),
            }))
        } else {
            Response::Unauthorized(json!({
                "error": "Invalid token!",
            }))
        }
    } else {
        Response::InternalServerError(json!({
            "error": "Failed database query.",
        }))
    }
}

#[options("/create")]
pub fn create_preflight() -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/verify/<_code>")]
pub fn verify_email_preflight(_code: String) -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/resend")]
pub fn resend_email_preflight() -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/login")]
pub fn login_preflight() -> Response {
    Response::Result(super::Status::Ok)
}
#[options("/token")]
pub fn token_preflight() -> Response {
    Response::Result(super::Status::Ok)
}
