use super::get_collection;
use super::guild::{Guild, fetch_guilds};
use super::channel::{Channel, fetch_channels};

use lru::LruCache;
use mongodb::bson::{doc, from_bson, Bson, DateTime};
use mongodb::options::FindOptions;
use rocket::http::{RawStr, Status};
use rocket::request::{self, FromParam, FromRequest, Request};
use rocket_contrib::json::JsonValue;
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

impl User {
    pub fn serialise(self, relationship: i32) -> JsonValue {
        if relationship == super::Relationship::SELF as i32 {
            json!({
                "id": self.id,
                "username": self.username,
                "display_name": self.display_name,
                "email": self.email,
                "verified": self.email_verification.verified,
            })
        } else {
            json!({
                "id": self.id,
                "username": self.username,
                "display_name": self.display_name,
                "relationship": relationship
            })
        }
    }

    pub fn find_guilds(&self) -> Result<Vec<String>, String> {
        let members = get_collection("members")
            .find(
                doc! {
                    "_id.user": &self.id
                },
                None
            ).map_err(|_| "Failed to fetch members.")?;
        
        Ok(members.into_iter()
            .filter_map(|x| match x {
                Ok(doc) => {
                    match doc.get_document("_id") {
                        Ok(id) => {
                            match id.get_str("guild") {
                                Ok(value) => Some(value.to_string()),
                                Err(_) => None
                            }
                        }
                        Err(_) => None
                    }
                }
                Err(_) => None
            })
            .collect())
    }

    pub fn find_dms(&self) -> Result<Vec<String>, String> {
        let channels = get_collection("channels")
            .find(
                doc! {
                    "recipients": &self.id
                },
                FindOptions::builder()
                    .projection(doc! { "_id": 1 })
                    .build()
            ).map_err(|_| "Failed to fetch channel ids.")?;
        
        Ok(channels.into_iter()
            .filter_map(|x| x.ok())
            .filter_map(|x| {
                match x.get_str("_id") {
                    Ok(value) => Some(value.to_string()),
                    Err(_) => None
                }
            })
            .collect())
    }

    pub fn create_payload(self) -> Result<JsonValue, String> {
        let v = vec![];
        let relations = self.relations.as_ref().unwrap_or(&v);
        
        let users: Vec<JsonValue> = fetch_users(
            &relations
                .iter()
                .map(|x| x.id.clone())
                .collect()
        )?
            .into_iter()
            .map(|x| {
                let id = x.id.clone();
                x.serialise(
                    relations.iter()
                        .find(|y| y.id == id)
                        .unwrap()
                        .status as i32
                )
            })
            .collect();

        let channels: Vec<JsonValue> = fetch_channels(&self.find_dms()?)?
            .into_iter()
            .map(|x| x.serialise())
            .collect();
        
        let guilds: Vec<JsonValue> = fetch_guilds(&self.find_guilds()?)?
            .into_iter()
            .map(|x| x.serialise())
            .collect();

        Ok(json!({
            "users": users,
            "channels": channels,
            "guilds": guilds,
            "user": self.serialise(super::Relationship::SELF as i32)
        }))
    }
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

pub fn fetch_users(ids: &Vec<String>) -> Result<Vec<User>, String> {
    let mut missing = vec![];
    let mut users = vec![];

    {
        if let Ok(mut cache) = CACHE.lock() {
            for id in ids {
                let existing = cache.get(id);

                if let Some(user) = existing {
                    users.push((*user).clone());
                } else {
                    missing.push(id);
                }
            }
        } else {
            return Err("Failed to lock cache.".to_string());
        }
    }

    if missing.len() == 0 {
        return Ok(users);
    }

    let col = get_collection("users");
    if let Ok(result) = col.find(doc! { "_id": { "$in": missing } }, None) {
        for item in result {
            let mut cache = CACHE.lock().unwrap();
            if let Ok(doc) = item {
                if let Ok(user) = from_bson(Bson::Document(doc)) as Result<User, _> {
                    cache.put(user.id.clone(), user.clone());
                    users.push(user);
                } else {
                    return Err("Failed to deserialize user!".to_string());
                }
            } else {
                return Err("Failed to fetch user.".to_string());
            }
        }

        Ok(users)
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
