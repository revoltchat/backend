use super::get_collection;

use serde::{Deserialize, Serialize};
use rocket::request::FromParam;
use std::sync::{Arc, Mutex};
use mongodb::bson::{Bson, doc, from_bson};
use rocket::http::RawStr;
use lru::LruCache;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemberRef {
    pub guild: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberRef,
    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invite {
    pub code: String,
    pub creator: String,
    pub channel: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ban {
    pub id: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Guild {
    #[serde(rename = "_id")]
    pub id: String,
    // pub nonce: String, used internally
    pub name: String,
    pub description: String,
    pub owner: String,

    pub invites: Vec<Invite>,
    pub bans: Vec<Ban>,

    pub default_permissions: u32,
}

lazy_static! {
    static ref CACHE: Arc<Mutex<LruCache<String, Guild>>> = Arc::new(Mutex::new(LruCache::new(4_000_000)));
}

pub fn fetch_guild(id: &str) -> Result<Option<Guild>, String> {
    {
        if let Ok(mut cache) = CACHE.lock() {
            let existing = cache.get(&id.to_string());

            if let Some(guild) = existing {
                return Ok(Some((*guild).clone()));
            }
        } else {
            return Err("Failed to lock cache.".to_string());
        }
    }

    let col = get_collection("guilds");
    if let Ok(result) = col.find_one(doc! { "_id": id }, None) {
        if let Some(doc) = result {
            if let Ok(guild) = from_bson(Bson::Document(doc)) as Result<Guild, _> {
                let mut cache = CACHE.lock().unwrap();
                cache.put(id.to_string(), guild.clone());
    
                Ok(Some(guild))
            } else {
                Err("Failed to deserialize guild!".to_string())
            }
        } else {
            Ok(None)
        }
    } else {
        Err("Failed to fetch guild from database.".to_string())
    }
}

impl<'r> FromParam<'r> for Guild {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        if let Ok(result) = fetch_guild(param) {
            if let Some(channel) = result {
                Ok(channel)
            } else {
                Err(param)
            }
        } else {
            Err(param)
        }
    }
}

pub fn get_member(guild_id: &String, member: &String) -> Option<Member> {
    if let Ok(result) = get_collection("members").find_one(
        doc! {
            "_id.guild": &guild_id,
            "_id.user": &member,
        },
        None,
    ) {
        if let Some(doc) = result {
            Some(from_bson(Bson::Document(doc)).expect("Failed to unwrap member."))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_invite<U: Into<Option<String>>>(
    code: &String,
    user: U,
) -> Option<(String, String, Invite)> {
    let mut doc = doc! {
        "invites": {
            "$elemMatch": {
                "code": &code
            }
        }
    };

    if let Some(user_id) = user.into() {
        doc.insert(
            "bans",
            doc! {
                "$not": {
                    "$elemMatch": {
                        "id": user_id
                    }
                }
            },
        );
    }

    if let Ok(result) = get_collection("guilds").find_one(
        doc,
        mongodb::options::FindOneOptions::builder()
            .projection(doc! {
                "_id": 1,
                "name": 1,
                "invites.$": 1,
            })
            .build(),
    ) {
        if let Some(doc) = result {
            let invite = doc
                .get_array("invites")
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .as_document()
                .unwrap();

            Some((
                doc.get_str("_id").unwrap().to_string(),
                doc.get_str("name").unwrap().to_string(),
                from_bson(Bson::Document(invite.clone())).unwrap(),
            ))
        } else {
            None
        }
    } else {
        None
    }
}
