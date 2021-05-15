use hive_pubsub::PubSub;
use rauth::auth::Session;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

use super::hive::{get_hive, subscribe_if_exists};
use crate::database::*;

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
#[serde(tag = "type")]
pub enum ClientboundNotification {
    Error(WebSocketError),
    Authenticated,
    Ready {
        users: Vec<User>,
        channels: Vec<Channel>,
    },

    Message(Message),
    MessageUpdate {
        id: String,
        data: JsonValue,
    },
    MessageDelete {
        id: String,
    },

    ChannelCreate(Channel),
    ChannelUpdate {
        id: String,
        data: JsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveChannelField>,
    },
    ChannelGroupJoin {
        id: String,
        user: String,
    },
    ChannelGroupLeave {
        id: String,
        user: String,
    },
    ChannelDelete {
        id: String,
    },
    ChannelStartTyping {
        id: String,
        user: String,
    },
    ChannelStopTyping {
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
}

impl ClientboundNotification {
    pub fn publish(self, topic: String) {
        async_std::task::spawn(async move {
            prehandle_hook(&self); // ! TODO: this should be moved to pubsub
            hive_pubsub::backend::mongo::publish(get_hive(), &topic, self)
                .await
                .ok();
        });
    }
}

pub fn prehandle_hook(notification: &ClientboundNotification) {
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
            }
        }
        ClientboundNotification::UserRelationship { id, user, status } => {
            if status != &RelationshipStatus::None {
                subscribe_if_exists(id.clone(), user.id.clone()).ok();
            }
        }
        _ => {}
    }
}

pub fn posthandle_hook(notification: &ClientboundNotification) {
    match &notification {
        ClientboundNotification::ChannelDelete { id } => {
            get_hive().hive.drop_topic(&id).ok();
        }
        ClientboundNotification::UserRelationship { id, user, status } => {
            if status == &RelationshipStatus::None {
                get_hive()
                    .hive
                    .unsubscribe(&id.to_string(), &user.id.to_string())
                    .ok();
            }
        }
        ClientboundNotification::ChannelGroupLeave { id, user } => {
            get_hive()
                .hive
                .unsubscribe(&user.to_string(), &id.to_string())
                .ok();
        }
        _ => {}
    }
}
