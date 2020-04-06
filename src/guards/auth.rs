use rocket::http::{RawStr, Status};
use rocket::request::{self, FromParam, FromRequest, Request};
use rocket::Outcome;

use bson::{bson, doc, from_bson};

use crate::database;
use database::user::User;

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
                    Outcome::Success(
                        from_bson(bson::Bson::Document(user)).expect("Failed to unwrap user."),
                    )
                } else {
                    Outcome::Failure((Status::Forbidden, AuthError::Invalid))
                }
            }
            _ => Outcome::Failure((Status::BadRequest, AuthError::BadCount)),
        }
    }
}

impl<'r> FromParam<'r> for User {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let col = database::get_db().collection("users");
        let result = col
            .find_one(doc! { "_id": param.to_string() }, None)
            .unwrap();

        if let Some(user) = result {
            Ok(from_bson(bson::Bson::Document(user)).expect("Failed to unwrap user."))
        } else {
            Err(param)
        }
    }
}
