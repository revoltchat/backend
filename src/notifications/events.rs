use hive_pubsub::PubSub;
use mongodb::bson::doc;
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

use super::hive::{get_hive, subscribe_if_exists};
use crate::{database::*, util::result::Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "error")]
pub enum WebSocketError {
    LabelMe,
    InternalError { at: String },
    InvalidSession,
    OnboardingNotFinished,
    AlreadyAuthenticated,
    MalformedData { msg: String },
}

#[derive(Deserialize, Debug)]
pub struct Auth {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Ping {
    Binary(Vec<u8>),
    Number(usize)
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServerboundNotification {
    Authenticate(Auth),
    BeginTyping { channel: String },
    EndTyping { channel: String },
    Ping { data: Ping, responded: Option<()> },
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
pub enum RemoveRoleField {
    Colour,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoveMemberField {
    Nickname,
    Avatar,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RemoveBotField {
    InteractionsURL,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ClientboundNotification {
    Error(WebSocketError),
    Authenticated,
    Ready {
        users: Vec<User>,
        servers: Vec<Server>,
        channels: Vec<Channel>,
        members: Vec<Member>,
    },
    Pong { data: Ping },

    Message(Message),
    MessageUpdate {
        id: String,
        channel: String,
        data: Value,
    },
    MessageDelete {
        id: String,
        channel: String,
    },

    ChannelCreate(Channel),
    ChannelUpdate {
        id: String,
        data: Value,
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
        message_id: String,
    },

    ServerUpdate {
        id: String,
        data: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveServerField>,
    },
    ServerDelete {
        id: String,
    },
    ServerMemberUpdate {
        id: MemberCompositeKey,
        data: Value,
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
    ServerRoleUpdate {
        id: String,
        role_id: String,
        data: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveRoleField>,
    },
    ServerRoleDelete {
        id: String,
        role_id: String,
    },

    UserUpdate {
        id: String,
        data: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        clear: Option<RemoveUserField>,
    },
    UserRelationship {
        id: String,
        user: User,
        status: RelationshipStatus,
    },
    UserSettingsUpdate {
        id: String,
        update: Value,
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

    pub fn publish_as_user(self, user: String) {
        self.clone().publish(user.clone());

        async_std::task::spawn(async move {
            if let Ok(server_ids) = User::fetch_server_ids(&user).await {
                for server in server_ids {
                    self.clone().publish(server.clone());
                }
            }
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
                Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => {
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
