use bson::{bson, doc, from_bson, Document};
use mongodb::options::FindOneOptions;
use rocket::http::{RawStr, Status};
use rocket::request::{self, FromParam, FromRequest, Request};
use rocket::Outcome;
use serde::{Deserialize, Serialize};

use crate::database;
use database::user::{User, UserRelationship};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserRef {
    pub id: String,
    pub username: String,
    pub email_verified: bool,
}

impl UserRef {
    pub fn from(id: String) -> Option<UserRef> {
        match database::get_collection("users").find_one(
            doc! { "_id": id },
            FindOneOptions::builder()
                .projection(doc! {
                    "_id": 1,
                    "username": 1,
                    "email_verification.verified": 1,
                })
                .build(),
        ) {
            Ok(result) => match result {
                Some(doc) => Some(UserRef {
                    id: doc.get_str("_id").unwrap().to_string(),
                    username: doc.get_str("username").unwrap().to_string(),
                    email_verified: doc
                        .get_document("email_verification")
                        .unwrap()
                        .get_bool("verified")
                        .unwrap(),
                }),
                None => None,
            },
            Err(_) => None,
        }
    }

    pub fn fetch_data(&self, projection: Document) -> Option<Document> {
        database::get_collection("users")
            .find_one(
                doc! { "_id": &self.id },
                FindOneOptions::builder().projection(projection).build(),
            )
            .expect("Failed to fetch user from database.")
    }

    pub fn fetch_relationships(&self) -> Option<Vec<UserRelationship>> {
        let user = database::get_collection("users")
            .find_one(
                doc! { "_id": &self.id },
                FindOneOptions::builder()
                    .projection(doc! { "relations": 1 })
                    .build(),
            )
            .expect("Failed to fetch user relationships from database.")
            .expect("Missing user document.");

        if let Ok(arr) = user.get_array("relations") {
            let mut relationships = vec![];
            for item in arr {
                relationships.push(from_bson(item.clone()).unwrap());
            }

            Some(relationships)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for UserRef {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("x-auth-token").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Forbidden, AuthError::Missing)),
            1 => {
                let key = keys[0];
                let result = database::get_collection("users")
                    .find_one(
                        doc! { "access_token": key },
                        FindOneOptions::builder()
                            .projection(doc! {
                                "_id": 1,
                                "username": 1,
                                "email_verification.verified": 1,
                            })
                            .build(),
                    )
                    .unwrap();

                if let Some(user) = result {
                    Outcome::Success(UserRef {
                        id: user.get_str("_id").unwrap().to_string(),
                        username: user.get_str("username").unwrap().to_string(),
                        email_verified: user
                            .get_document("email_verification")
                            .unwrap()
                            .get_bool("verified")
                            .unwrap(),
                    })
                } else {
                    Outcome::Failure((Status::Forbidden, AuthError::Invalid))
                }
            }
            _ => Outcome::Failure((Status::BadRequest, AuthError::BadCount)),
        }
    }
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

impl<'r> FromParam<'r> for UserRef {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        if let Some(user) = UserRef::from(param.to_string()) {
            Ok(user)
        } else {
            Err(param)
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
