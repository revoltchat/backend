use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};

use bson::{ bson, doc, ordered::OrderedDocument, oid::ObjectId };
use crate::database;

pub struct User(
	pub ObjectId,
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
			0 => Outcome::Failure((Status::BadRequest, AuthError::Missing)),
			1 => {
				let key = keys[0];
				let col = database::get_db().collection("users");
				let result = col.find_one(Some( doc! { "auth_token": key } ), None).unwrap();

				if let Some(user) = result {
					Outcome::Success(User(
						user.get_object_id("_id").unwrap().clone(),
						user.get_str("username").unwrap().to_owned(),
						user
					))
				} else {
					Outcome::Failure((Status::BadRequest, AuthError::Invalid))
				}
			},
            _ => Outcome::Failure((Status::BadRequest, AuthError::BadCount)),
        }
    }
}