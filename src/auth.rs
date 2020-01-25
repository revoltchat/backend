use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};

use bson::{ bson, doc, ordered::OrderedDocument };
use ulid::Ulid;

use crate::database;

pub struct User(
	pub Ulid,
	pub String,
	pub OrderedDocument,
);

#[derive(Debug)]
pub enum AuthError {
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("x-auth-token").collect();
        match keys.len() {
			0 => Outcome::Failure((Status::Forbidden, AuthError::Missing)),
			1 => {
				let key = keys[0];
				let col = database::get_db().collection("users");
				let result = col.find_one(doc! { "access_token": key }, None).unwrap();

				if let Some(user) = result {
					Outcome::Success(User(
						Ulid::from_string(user.get_str("_id").unwrap()).unwrap(),
						user.get_str("username").unwrap().to_string(),
						user
					))
				} else {
					Outcome::Failure((Status::Forbidden, AuthError::Invalid))
				}
			},
            _ => Outcome::Failure((Status::BadRequest, AuthError::BadCount)),
        }
    }
}