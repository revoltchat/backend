use hive_pubsub::PubSub;
use rauth::auth::Session;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use super::hive::{get_hive, subscribe_if_exists};
use crate::database::*;

#[derive(Serialize, Deserialize, Debug, Snafu)]
#[serde(tag = "error")]
pub enum WebSocketError {
    #[snafu(display("This error has not been labelled."))]
    LabelMe,
    #[snafu(display("Internal server error."))]
    InternalError { at: String },
    #[snafu(display("Invalid session."))]
    InvalidSession,
    #[snafu(display("User hasn't completed onboarding."))]
    OnboardingNotFinished,
    #[snafu(display("Already authenticated with server."))]
    AlreadyAuthenticated,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServerboundNotification {
    Authenticate(Session),
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
    MessageUpdate(JsonValue),
    MessageDelete {
        id: String,
    },

    ChannelCreate(Channel),
    ChannelUpdate(JsonValue),
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

    UserRelationship {
        id: String,
        user: String,
        status: RelationshipStatus,
    },
    UserPresence {
        id: String,
        online: bool,
    },
}

impl ClientboundNotification {
    pub async fn publish(self, topic: String) -> Result<(), String> {
        prehandle_hook(&self); // ! TODO: this should be moved to pubsub
        hive_pubsub::backend::mongo::publish(get_hive(), &topic, self).await
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
        ClientboundNotification::ChannelGroupLeave { id, user } => {
            get_hive()
                .hive
                .unsubscribe(&user.to_string(), &id.to_string())
                .ok();
        }
        ClientboundNotification::UserRelationship { id, user, status } => {
            if status != &RelationshipStatus::None {
                subscribe_if_exists(id.clone(), user.clone()).ok();
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
                    .unsubscribe(&id.to_string(), &user.to_string())
                    .ok();
            }
        }
        _ => {}
    }
}
