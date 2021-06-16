use hive_pubsub::PubSub;
use mongodb::bson::doc;
use rauth::auth::Session;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

use super::hive::{get_hive, subscribe_if_exists};
use crate::{
    database::*,
    util::result::{Result},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoveUserField {
    ProfileContent,
    ProfileBackground,
    StatusText,
    Avatar,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoveChannelField {
    Icon,
    Description,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoveServerField {
    Icon,
    Banner,
    Description,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoveMemberField {
    Nickname,
    Avatar,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ClientboundNotification {
    Error(WebSocketError),
    Authenticated,
    Ready {
        users: Vec<User>,
        servers: Vec<Server>,
        channels: Vec<Channel>
    },

    Message(Message),
    MessageUpdate {
        id: String,
        channel: String,
        data: JsonValue
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
    ChannelAck {
        id: String,
        user: String,
        message_id: String
    },

    ServerUpdate {
        id: String,
        data: JsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveServerField>,
    },
    ServerDelete {
        id: String,
    },
    ServerMemberUpdate {
        id: MemberCompositeKey,
        data: JsonValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveMemberField>,
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
                    let members = Server::fetch_member_ids(server).await?;
                    for member in members {
                        subscribe_if_exists(member.clone(), channel_id.to_string()).ok();
                    }
                }
            }
        }
        ClientboundNotification::ServerMemberJoin { id, user } => {
            let server = Ref::from_unchecked(id.clone()).fetch_server().await?;

            subscribe_if_exists(user.clone(), id.clone()).ok();

            for channel in server.channels {
                subscribe_if_exists(user.clone(), channel).ok();
            }
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

pub async fn posthandle_hook(notification: &ClientboundNotification) {
    match &notification {
        ClientboundNotification::ChannelDelete { id } => {
            get_hive().hive.drop_topic(&id).ok();
        }
        ClientboundNotification::ChannelGroupLeave { id, user } => {
            get_hive().hive.unsubscribe(user, id).ok();
        }
        ClientboundNotification::ServerDelete { id } => {
            get_hive().hive.drop_topic(&id).ok();
        }
        ClientboundNotification::ServerMemberLeave { id, user } => {
            get_hive().hive.unsubscribe(user, id).ok();

            if let Ok(server) = Ref::from_unchecked(id.clone()).fetch_server().await {
                for channel in server.channels {
                    get_hive().hive.unsubscribe(user, &channel).ok();
                }
            }
        }
        ClientboundNotification::UserRelationship { id, user, status } => {
            if status == &RelationshipStatus::None {
                get_hive().hive.unsubscribe(id, &user.id).ok();
            }
        }
        _ => {}
    }
}
