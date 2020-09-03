use super::events::Notification;
use super::websocket;
use crate::database::get_collection;

use hive_pubsub::backend::mongo::{listen_thread, MongodbPubSub};
use hive_pubsub::PubSub;
use once_cell::sync::OnceCell;
use serde_json::to_string;
use log::{error, debug};

static HIVE: OnceCell<MongodbPubSub<String, String, Notification>> = OnceCell::new();

pub fn init_hive() {
    let hive = MongodbPubSub::new(
        |ids, notification| {
            if let Ok(data) = to_string(&notification) {
                debug!("Pushing out notification. {}", data);
                if let Err(err) = websocket::publish(ids, data) {
                    error!("Failed to publish notification through WebSocket! {}", err);
                }
            } else {
                error!("Failed to serialise notification.");
            }
        },
        get_collection("hive"),
    );

    listen_thread(hive.clone());

    if HIVE.set(hive).is_err() {
        panic!("Failed to set global pubsub instance.");
    }
}

pub fn publish(topic: &String, data: Notification) -> Result<(), String> {
    let hive = HIVE.get().expect("Global pubsub instance not available.");
    hive.publish(topic, data)
}

pub fn subscribe(user: String, topics: Vec<String>) -> Result<(), String> {
    let hive = HIVE.get().expect("Global pubsub instance not available.");
    for topic in topics {
        hive.subscribe(user.clone(), topic)?;
    }

    Ok(())
}

pub fn drop_user(user: &String) -> Result<(), String> {
    let hive = HIVE.get().expect("Global pubsub instance not available.");
    hive.drop_client(user)?;

    Ok(())
}

pub fn drop_topic(topic: &String) -> Result<(), String> {
    let hive = HIVE.get().expect("Global pubsub instance not available.");
    hive.drop_topic(topic)?;

    Ok(())
}
