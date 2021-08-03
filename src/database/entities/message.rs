use crate::util::variables::{USE_JANUARY, VAPID_PRIVATE_KEY};
use crate::{
    database::*,
    notifications::{events::ClientboundNotification, websocket::is_online},
    util::result::{Error, Result},
};

use futures::StreamExt;
use mongodb::options::UpdateOptions;
use mongodb::{
    bson::{doc, to_bson, DateTime},
    options::FindOptions,
};
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use web_push::{
    ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

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
        get_collection("messages")
            .insert_one(to_bson(&self).unwrap().as_document().unwrap().clone(), None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "message",
            })?;

        // ! FIXME: all this code is legitimately crap
        // ! rewrite when can be asked

        let ss = self.clone();
        let c_clone = channel.clone();
        async_std::task::spawn(async move {
            let mut set = if let Content::Text(text) = &ss.content {
                doc! {
                    "last_message": {
                        "_id": ss.id.clone(),
                        "author": ss.author.clone(),
                        "short": text.chars().take(128).collect::<String>()
                    }
                }
            } else {
                doc! {}
            };

            // ! MARK AS ACTIVE
            // ! FIXME: temp code
            let channels = get_collection("channels");
            match &c_clone {
                Channel::DirectMessage { id, .. } => {
                    set.insert("active", true);
                    channels
                        .update_one(
                            doc! { "_id": id },
                            doc! {
                                "$set": set
                            },
                            None,
                        )
                        .await
                        /*.map_err(|_| Error::DatabaseError {
                            operation: "update_one",
                            with: "channel",
                        })?;*/
                        .unwrap();
                }
                Channel::Group { id, .. } => {
                    if let Content::Text(_) = &ss.content {
                        channels
                            .update_one(
                                doc! { "_id": id },
                                doc! {
                                    "$set": set
                                },
                                None,
                            )
                            .await
                            /*.map_err(|_| Error::DatabaseError {
                                operation: "update_one",
                                with: "channel",
                            })?;*/
                            .unwrap();
                    }
                }
                Channel::TextChannel { id, .. } => {
                    if let Content::Text(_) = &ss.content {
                        channels
                            .update_one(
                                doc! { "_id": id },
                                doc! {
                                    "$set": {
                                        "last_message": &ss.id
                                    }
                                },
                                None,
                            )
                            .await
                            /*.map_err(|_| Error::DatabaseError {
                                operation: "update_one",
                                with: "channel",
                            })?;*/
                            .unwrap();
                    }
                }
                _ => {}
            }
        });

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
        }

        if process_embeds {
            self.process_embed();
        }

        let mentions = self.mentions.clone();
        let enc = serde_json::to_string(&self).unwrap();
        ClientboundNotification::Message(self).publish(channel.id().to_string());

        /*
           Web Push Test Code
        */
        let c_clone = channel.clone();
        async_std::task::spawn(async move {
            // Find all offline users.
            let mut target_ids = vec![];
            match &c_clone {
                Channel::DirectMessage { recipients, .. } | Channel::Group { recipients, .. } => {
                    for recipient in recipients {
                        if !is_online(recipient) {
                            target_ids.push(recipient.clone());
                        }
                    }
                }
                Channel::TextChannel { .. } => {
                    if let Some(mut mentions) = mentions {
                        target_ids.append(&mut mentions);
                    }
                }
                _ => {}
            }

            // Fetch their corresponding sessions.
            if let Ok(mut cursor) = get_collection("accounts")
                .find(
                    doc! {
                        "_id": {
                            "$in": target_ids
                        },
                        "sessions.subscription": {
                            "$exists": true
                        }
                    },
                    FindOptions::builder()
                        .projection(doc! { "sessions": 1 })
                        .build(),
                )
                .await
            {
                let mut subscriptions = vec![];
                while let Some(result) = cursor.next().await {
                    if let Ok(doc) = result {
                        if let Ok(sessions) = doc.get_array("sessions") {
                            for session in sessions {
                                if let Some(doc) = session.as_document() {
                                    if let Ok(sub) = doc.get_document("subscription") {
                                        let endpoint = sub.get_str("endpoint").unwrap().to_string();
                                        let p256dh = sub.get_str("p256dh").unwrap().to_string();
                                        let auth = sub.get_str("auth").unwrap().to_string();

                                        subscriptions
                                            .push(SubscriptionInfo::new(endpoint, p256dh, auth));
                                    }
                                }
                            }
                        }
                    }
                }

                if subscriptions.len() > 0 {
                    let client = WebPushClient::new();
                    let key =
                        base64::decode_config(VAPID_PRIVATE_KEY.clone(), base64::URL_SAFE).unwrap();

                    for subscription in subscriptions {
                        let mut builder = WebPushMessageBuilder::new(&subscription).unwrap();
                        let sig_builder = VapidSignatureBuilder::from_pem(
                            std::io::Cursor::new(&key),
                            &subscription,
                        )
                        .unwrap();
                        let signature = sig_builder.build().unwrap();
                        builder.set_vapid_signature(signature);
                        builder.set_payload(ContentEncoding::AesGcm, enc.as_bytes());
                        let m = builder.build().unwrap();
                        client.send(m).await.ok();
                    }
                }
            }
        });

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
        self.process_embed();

        Ok(())
    }

    pub fn process_embed(&self) {
        if !*USE_JANUARY {
            return;
        }

        if let Content::Text(text) = &self.content {
            // ! FIXME: re-write this at some point,
            // ! or just before we allow user generated embeds
            let id = self.id.clone();
            let content = text.clone();
            let channel = self.channel.clone();
            async_std::task::spawn(async move {
                if let Ok(embeds) = Embed::generate(content).await {
                    if let Ok(bson) = to_bson(&embeds) {
                        if let Ok(_) = get_collection("messages")
                            .update_one(
                                doc! {
                                    "_id": &id
                                },
                                doc! {
                                    "$set": {
                                        "embeds": bson
                                    }
                                },
                                None,
                            )
                            .await
                        {
                            ClientboundNotification::MessageUpdate {
                                id,
                                channel: channel.clone(),
                                data: json!({ "embeds": embeds }),
                            }
                            .publish(channel);
                        }
                    }
                }
            });
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
                .update_one(
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
                    operation: "update_one",
                    with: "attachment",
                })?;
        }

        Ok(())
    }
}
