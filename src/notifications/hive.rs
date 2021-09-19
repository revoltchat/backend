use std::sync::{Arc, Mutex};

use super::{events::ClientboundNotification, websocket};
use crate::redis::get_pool;
use crate::util::variables::REDIS_URI;

use futures::FutureExt;
use hive_pubsub::backend::redis::RedisPubSub;
use hive_pubsub::PubSub;
use log::{debug, error};
use once_cell::sync::OnceCell;
use serde_json::to_string;

type Hive<'a> = RedisPubSub<'a, String, String, ClientboundNotification>;
static HIVE: OnceCell<Hive<'static>> = OnceCell::new();

pub async fn init_hive() {
    let pubsub_con = redis::Client::open(REDIS_URI.to_string()).unwrap().get_async_connection().await.unwrap().into_pubsub();

    let hive = RedisPubSub::new(
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
        get_pool(),
        Arc::new(Mutex::new(pubsub_con))
    );

    if HIVE.set(hive).is_err() {
        panic!("Failed to set global pubsub instance.");
    }
}

pub async fn listen() {
    HIVE.get()
        .unwrap()
        .clone()
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

pub fn get_hive() -> &'static Hive<'static> {
    HIVE.get().unwrap()
}
