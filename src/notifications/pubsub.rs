use super::events::Notification;
use crate::database::get_collection;

use mongodb::bson::{doc, from_bson, to_bson, Bson};
use mongodb::options::{CursorType, FindOneOptions, FindOptions};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use ulid::Ulid;

use once_cell::sync::OnceCell;
static SOURCEID: OnceCell<String> = OnceCell::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct PubSubMessage {
    #[serde(rename = "_id")]
    id: String,
    source: String,

    user_recipients: Option<Vec<String>>,
    target_guild: Option<String>,
    data: Notification,
}

pub fn send_message(users: Option<Vec<String>>, guild: Option<String>, data: Notification) -> bool {
    let message = PubSubMessage {
        id: Ulid::new().to_string(),
        source: SOURCEID.get().unwrap().to_string(),
        user_recipients: users.into(),
        target_guild: guild.into(),
        data,
    };

    if get_collection("pubsub")
        .insert_one(
            to_bson(&message)
                .expect("Failed to serialize pubsub message.")
                .as_document()
                .expect("Failed to convert to a document.")
                .clone(),
            None,
        )
        .is_ok()
    {
        true
    } else {
        false
    }
}

pub fn launch_subscriber() {
    let source = Ulid::new().to_string();
    SOURCEID
        .set(source.clone())
        .expect("Failed to create and set source ID.");

    let pubsub = get_collection("pubsub");
    if let Ok(result) = pubsub.find_one(
        doc! {},
        FindOneOptions::builder().sort(doc! { "_id": -1 }).build(),
    ) {
        let query = if let Some(doc) = result {
            doc! { "_id": { "$gt": doc.get_str("_id").unwrap() } }
        } else {
            doc! {}
        };

        if let Ok(mut cursor) = pubsub.find(
            query,
            FindOptions::builder()
                .cursor_type(CursorType::TailableAwait)
                .no_cursor_timeout(true)
                .max_await_time(Duration::from_secs(1200))
                .build(),
        ) {
            loop {
                while let Some(item) = cursor.next() {
                    if let Ok(doc) = item {
                        if let Ok(message) =
                            from_bson(Bson::Document(doc)) as Result<PubSubMessage, _>
                        {
                            if &message.source != &source {
                                super::state::send_message(
                                    message.user_recipients,
                                    message.target_guild,
                                    message.data.serialize(),
                                );
                            }
                        } else {
                            eprintln!("Failed to deserialize pubsub message.");
                        }
                    } else {
                        eprintln!("Failed to unwrap a document from pubsub.");
                    }
                }
            }
        } else {
            eprintln!("Failed to open subscriber cursor.");
        }
    } else {
        eprintln!("Failed to fetch latest document from pubsub collection.");
    }
}
