use super::get_collection;
use crate::guards::channel::ChannelRef;
use crate::notifications;
use crate::notifications::events::message::Create;
use crate::notifications::events::Notification;
use crate::routes::channel::ChannelType;

use bson::{doc, to_bson, UtcDateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PreviousEntry {
    pub content: String,
    pub time: UtcDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub nonce: Option<String>,
    pub channel: String,
    pub author: String,

    pub content: String,
    pub edited: Option<UtcDateTime>,

    pub previous_content: Vec<PreviousEntry>,
}

// ? TODO: write global send message
// ? pub fn send_message();
// ? handle websockets?
impl Message {
    pub fn send(&self, target: &ChannelRef) -> bool {
        if get_collection("messages")
            .insert_one(to_bson(&self).unwrap().as_document().unwrap().clone(), None)
            .is_ok()
        {
            notifications::send_message_given_channel(
                Notification::message_create(Create {
                    id: self.id.clone(),
                    nonce: self.nonce.clone(),
                    channel: self.channel.clone(),
                    author: self.author.clone(),
                    content: self.content.clone(),
                }),
                &target,
            );

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
