use super::get_collection;

use lru::LruCache;
use mongodb::bson::{doc, from_bson, Bson, DateTime};
use rocket::http::{RawStr, Status};
use rocket::request::{self, FromParam, FromRequest, Request};
use rocket::Outcome;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserEmailVerification {
    pub verified: bool,
    pub target: Option<String>,
    pub expiry: Option<DateTime>,
    pub rate_limit: Option<DateTime>,
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserRelationship {
    pub id: String,
    pub status: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub access_token: Option<String>,
    pub email_verification: UserEmailVerification,
    pub relations: Option<Vec<UserRelationship>>,
}

lazy_static! {
    static ref CACHE: Arc<Mutex<LruCache<String, User>>> =
        Arc::new(Mutex::new(LruCache::new(4_000_000)));
}

pub fn fetch_user(id: &str) -> Result<Option<User>, String> {
    {
        if let Ok(mut cache) = CACHE.lock() {
            let existing = cache.get(&id.to_string());

            if let Some(user) = existing {
                return Ok(Some((*user).clone()));
            }
        } else {
            return Err("Failed to lock cache.".to_string());
        }
    }

    let col = get_collection("users");
    if let Ok(result) = col.find_one(doc! { "_id": id }, None) {
        if let Some(doc) = result {
            if let Ok(user) = from_bson(Bson::Document(doc)) as Result<User, _> {
                let mut cache = CACHE.lock().unwrap();
                cache.put(id.to_string(), user.clone());

                Ok(Some(user))
            } else {
                Err("Failed to deserialize user!".to_string())
            }
        } else {
            Ok(None)
        }
    } else {
        Err("Failed to fetch user from database.".to_string())
    }
}

#[derive(Debug)]
pub enum AuthError {
    Failed,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let u = request.headers().get("x-user").next(); 
        let t = request.headers().get("x-auth-token").next();

        if let Some(uid) = u {
            if let Some(token) = t {
                if let Ok(result) = fetch_user(uid) {
                    if let Some(user) = result {
                        if let Some(access_token) = &user.access_token {
                            if access_token == token {
                                Outcome::Success(user)
                            } else {
                                Outcome::Failure((Status::Forbidden, AuthError::Invalid))
                            }
                        } else {
                            Outcome::Failure((Status::Forbidden, AuthError::Invalid))
                        }
                    } else {
                        Outcome::Failure((Status::Forbidden, AuthError::Invalid))
                    }
                } else {
                    Outcome::Failure((Status::Forbidden, AuthError::Failed))
                }
            } else {
                Outcome::Failure((Status::Forbidden, AuthError::Missing))
            }
        } else {
            Outcome::Failure((Status::Forbidden, AuthError::Missing))
        }
    }
}

impl<'r> FromParam<'r> for User {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        if let Ok(result) = fetch_user(&param.to_string()) {
            if let Some(user) = result {
                Ok(user)
            } else {
                Err(param)
            }
        } else {
            Err(param)
        }
    }
}

use crate::notifications::events::Notification;

pub fn process_event(event: &Notification) {
    match event {
        Notification::user_friend_status(ev) => {
            let mut cache = CACHE.lock().unwrap();
            if let Some(user) = cache.peek_mut(&ev.id) {
                if let Some(relations) = user.relations.as_mut() {
                    if ev.status == 0 {
                        if let Some(pos) = relations.iter().position(|x| x.id == ev.user) {
                            relations.remove(pos);
                        }
                    } else {
                        if let Some(entry) = relations.iter_mut().find(|x| x.id == ev.user) {
                            entry.status = ev.status as u8;
                        } else {
                            relations.push(
                                UserRelationship {
                                    id: ev.id.clone(),
                                    status: ev.status as u8
                                }
                            );
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
