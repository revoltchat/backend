use futures::StreamExt;
use hive_pubsub::PubSub;
use rauth::auth::Session;
use mongodb::bson::{Document, doc};
use serde::{Deserialize, Serialize};
use rocket_contrib::json::JsonValue;

use super::hive::{get_hive, subscribe_if_exists};
use crate::{database::*, util::result::{Error, Result}};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "error")]
pub enum WebSocketError {
    LabelMe,
    InternalError { at: String },
    InvalidSession,
    OnboardingNotFinished,
    AlreadyAuthenticated,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServerboundNotification {
    Authenticate(Session),
    BeginTyping { channel: String },
    EndTyping { channel: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RemoveUserField {
    ProfileContent,
    ProfileBackground,
    StatusText,
    Avatar,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RemoveChannelField {
    Icon,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RemoveServerField {
    Icon,
    Banner,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientboundNotification {
    Error(WebSocketError),
    Authenticated,
    Ready {
        users: Vec<User>,
        servers: Vec<Server>,
        channels: Vec<Channel>,
    },

    Message(Message),
    MessageUpdate {
        id: String,
        channel: String,
        data: JsonValue,
    },
    MessageDelete {
        id: String,
        channel: String,
    },

    ChannelCreate(Channel),
    ChannelUpdate {
        id: String,
        data: JsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveChannelField>,
    },
    ChannelDelete {
        id: String,
    },
    ChannelGroupJoin {
        id: String,
        user: String,
    },
    ChannelGroupLeave {
        id: String,
        user: String,
    },
    ChannelStartTyping {
        id: String,
        user: String,
    },
    ChannelStopTyping {
        id: String,
        user: String,
    },

    ServerCreate(Server),
    ServerUpdate {
        id: String,
        data: JsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveServerField>,
    },
    ServerDelete {
        id: String,
    },
    ServerMemberJoin {
        id: String,
        user: String,
    },
    ServerMemberLeave {
        id: String,
        user: String,
    },

    UserUpdate {
        id: String,
        data: JsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveUserField>,
    },
    UserRelationship {
        id: String,
        user: User,
        status: RelationshipStatus,
    },
    UserPresence {
        id: String,
        online: bool,
    },
    UserSettingsUpdate {
        id: String,
        update: JsonValue,
    },
}

impl ClientboundNotification {
    pub fn publish(self, topic: String) {
        async_std::task::spawn(async move {
            prehandle_hook(&self).await.ok(); // ! FIXME: this should be moved to pubsub
            hive_pubsub::backend::mongo::publish(get_hive(), &topic, self)
                .await
                .ok();
        });
    }
}

pub async fn prehandle_hook(notification: &ClientboundNotification) -> Result<()> {
    match &notification {
        ClientboundNotification::ChannelGroupJoin { id, user } => {
            subscribe_if_exists(user.clone(), id.clone()).ok();
        }
        ClientboundNotification::ChannelCreate(channel) => {
            let channel_id = channel.id();
            match &channel {
                Channel::SavedMessages { user, .. } => {
                    subscribe_if_exists(user.clone(), channel_id.to_string()).ok();
                }
                Channel::DirectMessage { recipients, .. } | Channel::Group { recipients, .. } => {
                    for recipient in recipients {
                        subscribe_if_exists(recipient.clone(), channel_id.to_string()).ok();
                    }
                }
                Channel::TextChannel { server, .. } => {
                    // ! FIXME: write a better algorithm?
                    let members = get_collection("server_members")
                        .find(
                            doc! {
                                "_id.server": server
                            },
                            None,
                        )
                        .await
                        .map_err(|_| Error::DatabaseError {
                            operation: "find",
                            with: "server_members",
                        })?
                        .filter_map(async move |s| s.ok())
                        .collect::<Vec<Document>>()
                        .await
                        .into_iter()
                        .filter_map(|x| {
                            x.get_document("_id")
                                .ok()
                                .map(|i| i.get_str("user").ok().map(|x| x.to_string()))
                        })
                        .flatten()
                        .collect::<Vec<String>>();

                    for member in members {
                        subscribe_if_exists(member.clone(), channel_id.to_string()).ok();
                    }
                }
            }
        }
        ClientboundNotification::ServerMemberJoin { id, user } => {
            subscribe_if_exists(user.clone(), id.clone()).ok();
        }
        ClientboundNotification::ServerCreate(server) => {
            subscribe_if_exists(server.owner.clone(), server.id.clone()).ok();
        }
        ClientboundNotification::UserRelationship { id, user, status } => {
            if status != &RelationshipStatus::None {
                subscribe_if_exists(id.clone(), user.id.clone()).ok();
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn posthandle_hook(notification: &ClientboundNotification) {
    match &notification {
        ClientboundNotification::ChannelDelete { id } => {
            get_hive().hive.drop_topic(&id).ok();
        }
        ClientboundNotification::ChannelGroupLeave { id, user } => {
            get_hive()
                .hive
                .unsubscribe(&user.to_string(), &id.to_string())
                .ok();
        }
        ClientboundNotification::ServerDelete { id } => {
            get_hive().hive.drop_topic(&id).ok();
        }
        ClientboundNotification::ServerMemberLeave { id, user } => {
            get_hive()
                .hive
                .unsubscribe(&user.to_string(), &id.to_string())
                .ok();
        }
        ClientboundNotification::UserRelationship { id, user, status } => {
            if status == &RelationshipStatus::None {
                get_hive()
                    .hive
                    .unsubscribe(&id.to_string(), &user.id.to_string())
                    .ok();
            }
        }
        _ => {}
    }
}
