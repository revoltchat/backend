use super::get_collection;

use serde::{Deserialize, Serialize};
use rocket::request::FromParam;
use std::sync::{Arc, Mutex};
use mongodb::bson::{Bson, doc, from_bson};
use rocket::http::RawStr;
use lru::LruCache;

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
    static ref CACHE: Arc<Mutex<LruCache<String, Channel>>> = Arc::new(Mutex::new(LruCache::new(4_000_000)));
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
        return Ok(Some(channels))
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
                    return Err("Failed to deserialize channel!".to_string())
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

/*pub fn test() {
    use std::time::Instant;

    let now = Instant::now();
    let mut cache = CACHE.lock().unwrap();
    println!("I'm about to write 4 million entries to cache.");
    for i in 0..4_000_000 {
        let c = Channel {
            id: "potato".to_string(),
            channel_type: 0,
    
            active: None,
            last_message: None,
            description: None,
            guild: None,
            name: None,
            owner: None,
            recipients: None
        };

        cache.put(format!("{}", i), c);
    }

    println!("It took {} seconds, roughly {}ms per entry.", now.elapsed().as_secs_f64(), now.elapsed().as_millis() as f64 / 1_000_000.0);

    let now = Instant::now();
    println!("Now I'm going to read every entry and immediately dispose of it.");
    for i in 0..4_000_000 {
        cache.get(&format!("{}", i));
    }

    println!("It took {} seconds, roughly {}ms per entry.", now.elapsed().as_secs_f64(), now.elapsed().as_millis() as f64 / 1_000_000.0);
}*/
