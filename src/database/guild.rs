use super::channel::fetch_channels;
use super::get_collection;

use lru::LruCache;
use mongodb::bson::{doc, from_bson, Bson};
use rocket::http::RawStr;
use rocket::request::FromParam;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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

    pub channels: Vec<String>,
    pub invites: Vec<Invite>,
    pub bans: Vec<Ban>,

    pub default_permissions: u32,
}

impl Guild {
    pub fn serialise(self) -> JsonValue {
        json!({
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "owner": self.owner
        })
    }

    pub fn fetch_channels(&self) -> Result<Vec<super::channel::Channel>, String> {
        super::channel::fetch_channels(&self.channels)
    }

    pub fn seralise_with_channels(self) -> Result<JsonValue, String> {
        let channels = self
            .fetch_channels()?
            .into_iter()
            .map(|x| x.serialise())
            .collect();

        let mut value = self.serialise();
        value
            .as_object_mut()
            .unwrap()
            .insert("channels".to_string(), channels);
        Ok(value)
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct MemberKey(pub String, pub String);

lazy_static! {
    static ref CACHE: Arc<Mutex<LruCache<String, Guild>>> =
        Arc::new(Mutex::new(LruCache::new(4_000_000)));
    static ref MEMBER_CACHE: Arc<Mutex<LruCache<MemberKey, Member>>> =
        Arc::new(Mutex::new(LruCache::new(4_000_000)));
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
            dbg!(doc.to_string());
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

pub fn fetch_guilds(ids: &Vec<String>) -> Result<Vec<Guild>, String> {
    let mut missing = vec![];
    let mut guilds = vec![];

    {
        if let Ok(mut cache) = CACHE.lock() {
            for id in ids {
                let existing = cache.get(id);

                if let Some(guild) = existing {
                    guilds.push((*guild).clone());
                } else {
                    missing.push(id);
                }
            }
        } else {
            return Err("Failed to lock cache.".to_string());
        }
    }

    if missing.len() == 0 {
        return Ok(guilds);
    }

    let col = get_collection("guilds");
    if let Ok(result) = col.find(doc! { "_id": { "$in": missing } }, None) {
        for item in result {
            let mut cache = CACHE.lock().unwrap();
            if let Ok(doc) = item {
                dbg!(doc.to_string());
                if let Ok(guild) = from_bson(Bson::Document(doc)) as Result<Guild, _> {
                    cache.put(guild.id.clone(), guild.clone());
                    guilds.push(guild);
                } else {
                    return Err("Failed to deserialize guild!".to_string());
                }
            } else {
                return Err("Failed to fetch guild.".to_string());
            }
        }

        Ok(guilds)
    } else {
        Err("Failed to fetch channel from database.".to_string())
    }
}

pub fn serialise_guilds_with_channels(ids: &Vec<String>) -> Result<Vec<JsonValue>, String> {
    let guilds = fetch_guilds(&ids)?;
    let cids: Vec<String> = guilds.iter().flat_map(|x| x.channels.clone()).collect();

    let channels = fetch_channels(&cids)?;
    Ok(guilds
        .into_iter()
        .map(|x| {
            let id = x.id.clone();
            let mut obj = x.serialise();
            obj.as_object_mut().unwrap().insert(
                "channels".to_string(),
                channels
                    .iter()
                    .filter(|x| x.guild.is_some() && x.guild.as_ref().unwrap() == &id)
                    .map(|x| x.clone().serialise())
                    .collect(),
            );
            obj
        })
        .collect())
}

pub fn fetch_member(key: MemberKey) -> Result<Option<Member>, String> {
    {
        if let Ok(mut cache) = MEMBER_CACHE.lock() {
            let existing = cache.get(&key);

            if let Some(member) = existing {
                return Ok(Some((*member).clone()));
            }
        } else {
            return Err("Failed to lock cache.".to_string());
        }
    }

    let col = get_collection("members");
    if let Ok(result) = col.find_one(
        doc! {
            "_id.guild": &key.0,
            "_id.user": &key.1,
        },
        None,
    ) {
        if let Some(doc) = result {
            if let Ok(member) = from_bson(Bson::Document(doc)) as Result<Member, _> {
                let mut cache = MEMBER_CACHE.lock().unwrap();
                cache.put(key, member.clone());

                Ok(Some(member))
            } else {
                Err("Failed to deserialize member!".to_string())
            }
        } else {
            Ok(None)
        }
    } else {
        Err("Failed to fetch member from database.".to_string())
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

/*use crate::notifications::events::Notification;

pub fn process_event(event: &Notification) {
    match event {
        Notification::guild_channel_create(_ev) => {} // ? for later use
        Notification::guild_channel_delete(_ev) => {} // ? for later use
        Notification::guild_delete(ev) => {
            let mut cache = CACHE.lock().unwrap();
            cache.pop(&ev.id);
        }
        Notification::guild_user_join(ev) => {
            let mut cache = MEMBER_CACHE.lock().unwrap();
            cache.put(
                MemberKey(ev.id.clone(), ev.user.clone()),
                Member {
                    id: MemberRef {
                        guild: ev.id.clone(),
                        user: ev.user.clone(),
                    },
                    nickname: None,
                },
            );
        }
        Notification::guild_user_leave(ev) => {
            let mut cache = MEMBER_CACHE.lock().unwrap();
            cache.pop(&MemberKey(ev.id.clone(), ev.user.clone()));
        }
        _ => {}
    }
}*/
