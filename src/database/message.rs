use super::get_collection;
use crate::database::channel::Channel;
use crate::pubsub::hive;
use crate::routes::channel::ChannelType;

use mongodb::bson::from_bson;
use mongodb::bson::{doc, to_bson, Bson, DateTime};
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};

use log::warn;

#[derive(Serialize, Deserialize, Debug)]
pub struct PreviousEntry {
    pub content: String,
    pub time: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub nonce: Option<String>,
    pub channel: String,
    pub author: String,

    pub content: String,
    pub edited: Option<DateTime>,

    pub previous_content: Vec<PreviousEntry>,
}

// ? TODO: write global send message
// ? pub fn send_message();
// ? handle websockets?
impl Message {
    pub fn send(&self, target: &Channel) -> bool {
        if get_collection("messages")
            .insert_one(to_bson(&self).unwrap().as_document().unwrap().clone(), None)
            .is_ok()
        {
            /*notifications::send_message_given_channel(
                Notification::message_create(Create {
                    id: self.id.clone(),
                    nonce: self.nonce.clone(),
                    channel: self.channel.clone(),
                    author: self.author.clone(),
                    content: self.content.clone(),
                }),
                &target,
            );*/

            if hive::publish(
                &target.id,
                crate::pubsub::events::Notification::message_create(
                    crate::pubsub::events::message::Create {
                        id: self.id.clone(),
                        nonce: self.nonce.clone(),
                        channel: self.channel.clone(),
                        author: self.author.clone(),
                        content: self.content.clone(),
                    },
                ),
            )
            .is_err()
            {
                warn!("Saved message but couldn't send notification.");
            }

            let short_content: String = self.content.chars().take(24).collect();

            // !! this stuff can be async
            if target.channel_type == ChannelType::DM as u8
                || target.channel_type == ChannelType::GROUPDM as u8
            {
                let mut update = doc! {
                    "$set": {
                        "last_message": {
                            "id": &self.id,
                            "user_id": &self.author,
                            "short_content": short_content,
                        }
                    }
                };

                if target.channel_type == ChannelType::DM as u8 {
                    update
                        .get_document_mut("$set")
                        .unwrap()
                        .insert("active", true);
                }

                if get_collection("channels")
                    .update_one(doc! { "_id": &target.id }, update, None)
                    .is_ok()
                {
                    true
                } else {
                    false
                }
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl<'r> FromParam<'r> for Message {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let col = get_collection("messages");
        let result = col
            .find_one(doc! { "_id": param.to_string() }, None)
            .unwrap();

        if let Some(message) = result {
            Ok(from_bson(Bson::Document(message)).expect("Failed to unwrap message."))
        } else {
            Err(param)
        }
    }
}
