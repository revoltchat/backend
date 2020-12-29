use super::events::ClientboundNotification;
use crate::database::get_collection;

use hive_pubsub::backend::mongo::MongodbPubSub;
use hive_pubsub::PubSub;
use once_cell::sync::OnceCell;
use serde_json::to_string;
use futures::FutureExt;
use log::{error, debug};

static HIVE: OnceCell<MongodbPubSub<String, String, ClientboundNotification>> = OnceCell::new();

pub async fn init_hive() {
    let hive = MongodbPubSub::new(
        |_ids, notification| {
            if let Ok(data) = to_string(&notification) {
                debug!("Pushing out notification. {}", data);
                // ! FIXME: push to websocket
            } else {
                error!("Failed to serialise notification.");
            }
        },
        get_collection("pubsub"),
    );

    if HIVE.set(hive).is_err() {
        panic!("Failed to set global pubsub instance.");
    }
}

pub async fn listen() {
    HIVE.get()
        .unwrap()
        .listen()
        .fuse()
        .await
        .expect("Hive hit an error");
    
    dbg!("a");
}

pub fn publish(topic: &String, data: ClientboundNotification) -> Result<(), String> {
    let hive = HIVE.get().unwrap();
    hive.publish(topic, data)
}

pub fn subscribe(user: String, topics: Vec<String>) -> Result<(), String> {
    let hive = HIVE.get().unwrap();
    for topic in topics {
        hive.subscribe(user.clone(), topic)?;
    }

    Ok(())
}

pub fn drop_user(user: &String) -> Result<(), String> {
    let hive = HIVE.get().unwrap();
    hive.drop_client(user)?;

    Ok(())
}

pub fn drop_topic(topic: &String) -> Result<(), String> {
    let hive = HIVE.get().unwrap();
    hive.drop_topic(topic)?;

    Ok(())
}
