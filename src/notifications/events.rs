use rauth::auth::Session;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use super::hive::get_hive;
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
    MessageEdit(Message),
    MessageDelete {
        id: String,
    },

    ChannelCreate(Channel),
    ChannelGroupJoin {
        id: String,
        user: String,
    },
    ChannelGroupLeave {
        id: String,
        user: String,
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
        hive_pubsub::backend::mongo::publish(get_hive(), &topic, self).await
    }
}
