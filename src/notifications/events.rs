use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Notification {
    MessageCreate {
        id: String,
        nonce: Option<String>,
        channel: String,
        author: String,
        content: String,
    },

    MessageEdit {
        id: String,
        channel: String,
        author: String,
        content: String,
    },

    MessageDelete {
        id: String,
    },

    GroupUserJoin {
        id: String,
        user: String,
    },

    GroupUserLeave {
        id: String,
        user: String,
    },

    GuildUserJoin {
        id: String,
        user: String,
    },

    GuildUserLeave {
        id: String,
        user: String,
        banned: bool,
    },

    GuildChannelCreate {
        id: String,
        channel: String,
        name: String,
        description: String,
    },

    GuildChannelDelete {
        id: String,
        channel: String,
    },

    GuildDelete {
        id: String,
    },

    UserRelationship {
        id: String,
        user: String,
        status: i32,
    }
}
