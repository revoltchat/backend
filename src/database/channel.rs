use crate::database::get_collection;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use lru::LruCache;
use bson::{doc, from_bson};

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

pub fn fetch_channel(id: &str) -> Channel {
    {
        let mut cache = CACHE.lock().unwrap();
        let existing = cache.get(&id.to_string());

        if let Some(channel) = existing {
            return (*channel).clone();
        }
    }

    let col = get_collection("channels");
    let result = col.find_one(doc! { "_id": id }, None).unwrap();

    if let Some(doc) = result {
        let channel: Channel = from_bson(bson::Bson::Document(doc)).expect("Failed to unwrap channel.");

        let mut cache = CACHE.lock().unwrap();
        cache.put(id.to_string(), channel.clone());

        return channel;
    }

    panic!("Channel does not exist!");
}
