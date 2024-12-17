use authifier::AuthifierEvent;
use revolt_result::Error;
use serde::{Deserialize, Serialize};

use revolt_models::v0::{
    AppendMessage, Channel, ChannelUnread, Emoji, FieldsChannel, FieldsMember, FieldsMessage,
    FieldsRole, FieldsServer, FieldsUser, FieldsWebhook, Member, MemberCompositeKey, Message,
    PartialChannel, PartialMember, PartialMessage, PartialRole, PartialServer, PartialUser,
    PartialWebhook, RemovalIntention, Report, Server, User, UserSettings, Webhook,
};

use crate::Database;

/// Ping Packet
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Ping {
    Binary(Vec<u8>),
    Number(usize),
}

/// Fields provided in Ready payload
#[derive(PartialEq)]
pub enum ReadyPayloadFields {
    Users,
    Servers,
    Channels,
    Members,
    Emoji,

    UserSettings(Vec<String>),
    ChannelUnreads,
}

/// Protocol Events
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventV1 {
    /// Multiple events
    Bulk { v: Vec<EventV1> },
    /// Error event
    Error { data: Error },

    /// Successfully authenticated
    Authenticated,
    /// Logged out
    Logout,
    /// Basic data to cache
    Ready {
        #[serde(skip_serializing_if = "Option::is_none")]
        users: Option<Vec<User>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        servers: Option<Vec<Server>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channels: Option<Vec<Channel>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        members: Option<Vec<Member>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        emojis: Option<Vec<Emoji>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        user_settings: Option<UserSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_unreads: Option<Vec<ChannelUnread>>,
    },

    /// Ping response
    Pong { data: Ping },
    /// New message
    Message(Message),

    /// Update existing message
    MessageUpdate {
        id: String,
        channel: String,
        data: PartialMessage,
        #[serde(default)]
        clear: Vec<FieldsMessage>,
    },

    /// Append information to existing message
    MessageAppend {
        id: String,
        channel: String,
        append: AppendMessage,
    },

    /// Delete message
    MessageDelete { id: String, channel: String },

    /// New reaction to a message
    MessageReact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    /// Remove user's reaction from message
    MessageUnreact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    /// Remove a reaction from message
    MessageRemoveReaction {
        id: String,
        channel_id: String,
        emoji_id: String,
    },

    /// Bulk delete messages
    BulkMessageDelete { channel: String, ids: Vec<String> },

    /// New server
    ServerCreate {
        id: String,
        server: Server,
        channels: Vec<Channel>,
        emojis: Vec<Emoji>,
    },

    /// Update existing server
    ServerUpdate {
        id: String,
        data: PartialServer,
        #[serde(default)]
        clear: Vec<FieldsServer>,
    },

    /// Delete server
    ServerDelete { id: String },

    /// Update existing server member
    ServerMemberUpdate {
        id: MemberCompositeKey,
        data: PartialMember,
        #[serde(default)]
        clear: Vec<FieldsMember>,
    },

    /// User joins server
    ServerMemberJoin { id: String, user: String },

    /// User left server
    ServerMemberLeave {
        id: String,
        user: String,
        reason: RemovalIntention,
    },

    /// Server role created or updated
    ServerRoleUpdate {
        id: String,
        role_id: String,
        data: PartialRole,
        #[serde(default)]
        clear: Vec<FieldsRole>,
    },

    /// Server role deleted
    ServerRoleDelete { id: String, role_id: String },

    /// Update existing user
    UserUpdate {
        id: String,
        data: PartialUser,
        #[serde(default)]
        clear: Vec<FieldsUser>,
        event_id: Option<String>,
    },

    /// Relationship with another user changed
    UserRelationship { id: String, user: User },
    /// Settings updated remotely
    UserSettingsUpdate { id: String, update: UserSettings },

    /// User has been platform banned or deleted their account
    ///
    /// Clients should remove the following associated data:
    /// - Messages
    /// - DM Channels
    /// - Relationships
    /// - Server Memberships
    ///
    /// User flags are specified to explain why a wipe is occurring though not all reasons will necessarily ever appear.
    UserPlatformWipe { user_id: String, flags: i32 },
    /// New emoji
    EmojiCreate(Emoji),

    /// Delete emoji
    EmojiDelete { id: String },

    /// New report
    ReportCreate(Report),
    /// New channel
    ChannelCreate(Channel),

    /// Update existing channel
    ChannelUpdate {
        id: String,
        data: PartialChannel,
        #[serde(default)]
        clear: Vec<FieldsChannel>,
    },

    /// Delete channel
    ChannelDelete { id: String },

    /// User joins a group
    ChannelGroupJoin { id: String, user: String },

    /// User leaves a group
    ChannelGroupLeave { id: String, user: String },

    /// User started typing in a channel
    ChannelStartTyping { id: String, user: String },

    /// User stopped typing in a channel
    ChannelStopTyping { id: String, user: String },

    /// User acknowledged message in channel
    ChannelAck {
        id: String,
        user: String,
        message_id: String,
    },

    /// New webhook
    WebhookCreate(Webhook),

    /// Update existing webhook
    WebhookUpdate {
        id: String,
        data: PartialWebhook,
        remove: Vec<FieldsWebhook>,
    },

    /// Delete webhook
    WebhookDelete { id: String },

    /// Auth events
    Auth(AuthifierEvent),
}

impl EventV1 {
    /// Publish helper wrapper
    pub async fn p(self, channel: String) {
        #[cfg(not(debug_assertions))]
        redis_kiss::p(channel, self).await;

        #[cfg(debug_assertions)]
        info!("Publishing event to {channel}: {self:?}");

        #[cfg(debug_assertions)]
        redis_kiss::publish(channel, self).await.unwrap();
    }

    /// Publish user event
    pub async fn p_user(self, id: String, db: &Database) {
        self.clone().p(id.clone()).await;

        // TODO: this should be captured by member list in the future and not immediately fanned out to users
        if let Ok(members) = db.fetch_all_memberships(&id).await {
            for member in members {
                self.clone().server(member.id.server).await;
            }
        }
    }

    /// Publish private event
    pub async fn private(self, id: String) {
        self.p(format!("{id}!")).await;
    }

    /// Publish server member event
    pub async fn server(self, id: String) {
        self.p(format!("{id}u")).await;
    }

    /// Publish internal global event
    pub async fn global(self) {
        self.p("global".to_string()).await;
    }
}
