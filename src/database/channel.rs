use super::get_collection;

use lru::LruCache;
use mongodb::bson::{doc, from_bson, Bson};
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastMessage {
    // message id
    id: String,
    // author's id
    user_id: String,
    // truncated content with author's name prepended (for GDM / GUILD)
    short_content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: u8,

    // DM: whether the DM is active
    pub active: Option<bool>,
    // DM + GDM: last message in channel
    pub last_message: Option<LastMessage>,
    // DM + GDM: recipients for channel
    pub recipients: Option<Vec<String>>,
    // GDM: owner of group
    pub owner: Option<String>,
    // GUILD: channel parent
    pub guild: Option<String>,
    // GUILD + GDM: channel name
    pub name: Option<String>,
    // GUILD + GDM: channel description
    pub description: Option<String>,
}

lazy_static! {
    static ref CACHE: Arc<Mutex<LruCache<String, Channel>>> =
        Arc::new(Mutex::new(LruCache::new(4_000_000)));
}

pub fn fetch_channel(id: &str) -> Result<Option<Channel>, String> {
    {
        if let Ok(mut cache) = CACHE.lock() {
            let existing = cache.get(&id.to_string());

            if let Some(channel) = existing {
                return Ok(Some((*channel).clone()));
            }
        } else {
            return Err("Failed to lock cache.".to_string());
        }
    }

    let col = get_collection("channels");
    if let Ok(result) = col.find_one(doc! { "_id": id }, None) {
        if let Some(doc) = result {
            if let Ok(channel) = from_bson(Bson::Document(doc)) as Result<Channel, _> {
                let mut cache = CACHE.lock().unwrap();
                cache.put(id.to_string(), channel.clone());

                Ok(Some(channel))
            } else {
                Err("Failed to deserialize channel!".to_string())
            }
        } else {
            Ok(None)
        }
    } else {
        Err("Failed to fetch channel from database.".to_string())
    }
}

pub fn fetch_channels(ids: &Vec<String>) -> Result<Option<Vec<Channel>>, String> {
    let mut missing = vec![];
    let mut channels = vec![];

    {
        if let Ok(mut cache) = CACHE.lock() {
            for gid in ids {
                let existing = cache.get(gid);

                if let Some(channel) = existing {
                    channels.push((*channel).clone());
                } else {
                    missing.push(gid);
                }
            }
        } else {
            return Err("Failed to lock cache.".to_string());
        }
    }

    if missing.len() == 0 {
        return Ok(Some(channels));
    }

    let col = get_collection("channels");
    if let Ok(result) = col.find(doc! { "_id": { "$in": missing } }, None) {
        for item in result {
            let mut cache = CACHE.lock().unwrap();
            if let Ok(doc) = item {
                if let Ok(channel) = from_bson(Bson::Document(doc)) as Result<Channel, _> {
                    cache.put(channel.id.clone(), channel.clone());
                    channels.push(channel);
                } else {
                    return Err("Failed to deserialize channel!".to_string());
                }
            } else {
                return Err("Failed to fetch channel.".to_string());
            }
        }

        Ok(Some(channels))
    } else {
        Err("Failed to fetch channel from database.".to_string())
    }
}

impl<'r> FromParam<'r> for Channel {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        if let Ok(result) = fetch_channel(param) {
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

use crate::notifications::events::Notification;

pub fn process_event(event: &Notification) {
    match event {
        Notification::group_user_join(ev) => {
            let mut cache = CACHE.lock().unwrap();
            if let Some(channel) = cache.peek_mut(&ev.id) {
                channel.recipients.as_mut().unwrap().push(ev.user.clone());
            }
        }
        Notification::group_user_leave(ev) => {
            let mut cache = CACHE.lock().unwrap();
            if let Some(channel) = cache.peek_mut(&ev.id) {
                let recipients = channel.recipients.as_mut().unwrap();
                if let Some(pos) = recipients.iter().position(|x| *x == ev.user) {
                    recipients.remove(pos);
                }
            }
        }
        Notification::guild_channel_create(ev) => {
            let mut cache = CACHE.lock().unwrap();
            cache.put(
                ev.id.clone(),
                Channel {
                    id: ev.channel.clone(),
                    channel_type: 2,
                    active: None,
                    last_message: None,
                    recipients: None,
                    owner: None,
                    guild: Some(ev.id.clone()),
                    name: Some(ev.name.clone()),
                    description: Some(ev.description.clone()),
                },
            );
        }
        Notification::guild_channel_delete(ev) => {
            let mut cache = CACHE.lock().unwrap();
            cache.pop(&ev.channel);
        }
        _ => {}
    }
}
