use crate::util::variables::{USE_JANUARY, VAPID_PRIVATE_KEY, PUBLIC_URL};
use crate::{
    database::*,
    notifications::{events::ClientboundNotification, websocket::is_online},
    util::result::{Error, Result},
};

use futures::StreamExt;
use mongodb::options::UpdateOptions;
use mongodb::{
    bson::{doc, to_bson, DateTime, Document},
};
use rauth::entities::{Model, Session};
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use web_push::{ContentEncoding, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushNotification {
    pub author: String,
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    pub body: String,
    pub tag: String,
    pub timestamp: u64,
}

impl PushNotification {
    pub async fn new(msg: Message, channel: &Channel) -> Self {
        let author = Ref::from_unchecked(msg.author.clone())
            .fetch_user()
            .await;

        let (author, avatar) = if let Ok(author) = author {
            (Some(author.username), author.avatar)
        } else {
            (None, None)
        };

        let icon = if let Some(avatar) = avatar {
            avatar.get_autumn_url()
        } else {
            format!("{}/users/{}/default_avatar", PUBLIC_URL.as_str(), msg.author)
        };

        let image = msg.attachments.map_or(None, |attachments| {
            attachments
                .first()
                .map_or(None, |v| Some(v.get_autumn_url()))
        });

        let body = match msg.content {
            Content::Text(body) => body,
            Content::SystemMessage(sys_msg) => sys_msg.into()
        };

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system time should be valid")
            .as_secs();

        Self {
            author: author.unwrap_or_else(|| "Unknown".into()),
            icon,
            image,
            body,
            tag: channel.id().to_string(),
            timestamp,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SystemMessage {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "user_added")]
    UserAdded { id: String, by: String },
    #[serde(rename = "user_remove")]
    UserRemove { id: String, by: String },
    #[serde(rename = "user_joined")]
    UserJoined { id: String },
    #[serde(rename = "user_left")]
    UserLeft { id: String },
    #[serde(rename = "user_kicked")]
    UserKicked { id: String },
    #[serde(rename = "user_banned")]
    UserBanned { id: String },
    #[serde(rename = "channel_renamed")]
    ChannelRenamed { name: String, by: String },
    #[serde(rename = "channel_description_changed")]
    ChannelDescriptionChanged { by: String },
    #[serde(rename = "channel_icon_changed")]
    ChannelIconChanged { by: String },
}

impl Into<String> for SystemMessage {
    fn into(self) -> String {
        match self {
            SystemMessage::Text { content } => content,
            SystemMessage::UserAdded { .. } => "User added to the channel.".to_string(),
            SystemMessage::UserRemove { .. } => "User removed from the channel.".to_string(),
            SystemMessage::UserJoined { .. } => "User joined the channel.".to_string(),
            SystemMessage::UserLeft { .. } => "User left the channel.".to_string(),
            SystemMessage::UserKicked { .. } => "User kicked from the channel.".to_string(),
            SystemMessage::UserBanned { .. } => "User banned from the channel.".to_string(),
            SystemMessage::ChannelRenamed { .. } => "Channel renamed.".to_string(),
            SystemMessage::ChannelDescriptionChanged { .. } => "Channel description changed.".to_string(),
            SystemMessage::ChannelIconChanged { .. } => "Channel icon changed.".to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    SystemMessage(SystemMessage),
}

impl Content {
    pub async fn send_as_system(self, target: &Channel) -> Result<()> {
        Message::create(
            "00000000000000000000000000".to_string(),
            target.id().to_string(),
            self,
            None,
            None
        )
        .publish(&target, false)
        .await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub channel: String,
    pub author: String,

    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<File>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<String>>
}

impl Message {
    pub fn create(
        author: String,
        channel: String,
        content: Content,
        mentions: Option<Vec<String>>,
        replies: Option<Vec<String>>,
    ) -> Message {
        Message {
            id: Ulid::new().to_string(),
            nonce: None,
            channel,
            author,
            content,
            attachments: None,
            edited: None,
            embeds: None,
            mentions,
            replies
        }
    }

    pub async fn publish(self, channel: &Channel, process_embeds: bool) -> Result<()> {
        // Publish message event
        ClientboundNotification::Message(self.clone())
            .publish(channel.id().to_string());

        // Commit message to database
        get_collection("messages")
            .insert_one(to_bson(&self).unwrap().as_document().unwrap().clone(), None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "message",
            })?;

        // spawn task_queue ( process embeds )
        if process_embeds {
            self.process_embed().await;
        }

        // spawn task_queue ( update last_message_id )
        match channel {
            Channel::DirectMessage { id, .. } =>
                crate::task_queue::task_last_message_id::queue(id.clone(), self.id.clone(), true).await,
            Channel::Group { id, .. } | Channel::TextChannel { id, .. } =>
                crate::task_queue::task_last_message_id::queue(id.clone(), self.id.clone(), false).await,
            _ => {}
        }

        // if mentions {
        //  spawn task_queue ( update channel_unreads )
        // }
        /*if let Some(mentions) = &self.mentions {

        }*/

        // if (channel => DM | Group) | mentions {
        //  spawn task_queue ( web push )
        // }
        let mut target_ids = vec![];
        match &channel {
            Channel::DirectMessage { recipients, .. } | Channel::Group { recipients, .. } => {
                for recipient in recipients {
                    if !is_online(recipient) {
                        target_ids.push(recipient.clone());
                    }
                }
            }
            Channel::TextChannel { .. } => {
                if let Some(mentions) = &self.mentions {
                    target_ids.append(&mut mentions.clone());
                }
            }
            _ => {}
        }

        if target_ids.len() > 0 {
            if let Ok(payload) = serde_json::to_string(&PushNotification::new(self, &channel).await) {
                crate::task_queue::task_web_push::queue(target_ids, payload).await;
            }
        }

        /*

        // ! FIXME: also temp code
        // ! THIS ADDS ANY MENTIONS
        if let Some(mentions) = &self.mentions {
            let message = self.id.clone();
            let channel = self.channel.clone();
            let mentions = mentions.clone();
            async_std::task::spawn(async move {
                get_collection("channel_unreads")
                    .update_many(
                        doc! {
                            "_id.channel": channel,
                            "_id.user": {
                                "$in": mentions
                            }
                        },
                        doc! {
                            "$push": {
                                "mentions": message
                            }
                        },
                        UpdateOptions::builder().upsert(true).build(),
                    )
                    .await
                    /*.map_err(|_| Error::DatabaseError {
                        operation: "update_many",
                        with: "channel_unreads",
                    })?;*/
                    .unwrap();
            });
        }*/

        Ok(())
    }

    pub async fn publish_update(self, data: Value) -> Result<()> {
        let channel = self.channel.clone();
        ClientboundNotification::MessageUpdate {
            id: self.id.clone(),
            channel: self.channel.clone(),
            data,
        }
        .publish(channel);

        self.process_embed().await;
        Ok(())
    }

    pub async fn process_embed(&self) {
        if !*USE_JANUARY {
            return;
        }

        if let Content::Text(text) = &self.content {
            crate::task_queue::task_process_embeds::queue(self.channel.clone(), self.id.clone(), text.clone()).await;
        }
    }

    pub async fn delete(&self) -> Result<()> {
        if let Some(attachments) = &self.attachments {
            for attachment in attachments {
                attachment.delete().await?;
            }
        }

        get_collection("messages")
            .delete_one(
                doc! {
                    "_id": &self.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "message",
            })?;

        let channel = self.channel.clone();
        ClientboundNotification::MessageDelete {
            id: self.id.clone(),
            channel: self.channel.clone(),
        }
        .publish(channel);

        if let Some(attachments) = &self.attachments {
            let attachment_ids: Vec<String> =
                attachments.iter().map(|f| f.id.to_string()).collect();
            get_collection("attachments")
                .update_many(
                    doc! {
                        "_id": {
                            "$in": attachment_ids
                        }
                    },
                    doc! {
                        "$set": {
                            "deleted": true
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_many",
                    with: "attachment",
                })?;
        }

        Ok(())
    }
}

async fn update_channels_last_message(channels: &Collection, channel_id: &String, set: &Document) {
    channels
        .update_one(
            doc! { "_id": channel_id },
            doc! { "$set": set },
            None,
        )
        .await
        .expect("Server should not run with no, or a corrupted db");

}
