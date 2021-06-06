use super::{events::ClientboundNotification, websocket};
use crate::database::*;

use futures::FutureExt;
use hive_pubsub::backend::mongo::MongodbPubSub;
use hive_pubsub::PubSub;
use log::{debug, error};
use once_cell::sync::OnceCell;
use serde_json::to_string;

type Hive = MongodbPubSub<String, String, ClientboundNotification>;
static HIVE: OnceCell<Hive> = OnceCell::new();

pub async fn init_hive() {
    let hive = MongodbPubSub::new(
        |ids, notification: ClientboundNotification| {
            let notif = notification.clone();
            async_std::task::spawn(async move {
                super::events::posthandle_hook(&notif).await;
            });

            if let Ok(data) = to_string(&notification) {
                debug!("Pushing out notification. {}", data);
                websocket::publish(ids, notification);
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
}

pub fn subscribe_multiple(user: String, topics: Vec<String>) -> Result<(), String> {
    let hive = HIVE.get().unwrap();
    for topic in topics {
        hive.subscribe(user.clone(), topic)?;
    }

    Ok(())
}

pub fn subscribe_if_exists(user: String, topic: String) -> Result<(), String> {
    let hive = HIVE.get().unwrap();
    if hive.hive.map.lock().unwrap().get_left(&user).is_some() {
        hive.subscribe(user, topic)?;
    }

    Ok(())
}

pub fn get_hive() -> &'static Hive {
    HIVE.get().unwrap()
}
