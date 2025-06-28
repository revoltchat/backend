use std::sync::Arc;

use async_std::sync::Mutex;
use authifier::AuthifierEvent;
use lapin::{
    options::QueueDeclareOptions,
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable},
};
use revolt_config::config;
use revolt_result::Error;
use serde::{Deserialize, Serialize};

use revolt_models::v0::{
    AppendMessage, Channel, ChannelUnread, Emoji, FieldsChannel, FieldsMember, FieldsMessage,
    FieldsRole, FieldsServer, FieldsUser, FieldsWebhook, Member, MemberCompositeKey, Message,
    PartialChannel, PartialMember, PartialMessage, PartialRole, PartialServer, PartialUser,
    PartialWebhook, PolicyChange, RemovalIntention, Report, Server, User, UserSettings, Webhook,
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

        policy_changes: Vec<PolicyChange>,
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

    /// Server roles ranks updated
    ServerRoleRanksUpdate { id: String, ranks: Vec<String> },

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

// * not sure where to put this yet...
pub async fn create_client() -> lapin::Connection {
    let config = config().await;

    lapin::Connection::connect(
        &format!(
            "amqp://{}:{}@{}:{}/%2f",
            config.rabbit.username, config.rabbit.password, config.rabbit.host, config.rabbit.port
        ),
        Default::default(),
    )
    .await
    .unwrap()
}

pub async fn create_event_stream_channel(conn: lapin::Connection) -> lapin::Channel {
    static STREAM_NAME: &str = "revolt.events";

    let channel = conn.create_channel().await.unwrap();

    let mut args: FieldTable = Default::default();

    args.insert(
        // set queue type to stream
        "x-queue-type".into(),
        AMQPValue::LongString("stream".into()),
    );

    args.insert(
        // max. size of the stream
        "x-max-length-bytes".into(),
        AMQPValue::LongLongInt(5_000_000_000), // 5 GB
    );

    args.insert(
        // size of the Bloom filter
        "x-stream-filter-size-bytes".into(),
        AMQPValue::LongLongInt(26), // 26 B
    );

    channel
        .queue_declare(
            STREAM_NAME,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            args,
        )
        .await
        .unwrap();

    channel.basic_qos(100, Default::default()).await.unwrap();

    channel
}

/// simple connection pooling algorithm
pub async fn get_event_stream_channel() -> Arc<lapin::Channel> {
    static CLIENTS_PER_CONN: usize = 128;
    static CONNECTIONS: Mutex<Vec<Arc<lapin::Channel>>> = Mutex::new(Vec::new());

    let mut connections = CONNECTIONS.lock().await;
    connections.retain(|item| {
        if item.status().connected() {
            true
        } else {
            info!(
                "Dropping connection with status {:?}",
                item.status().state()
            );
            false
        }
    });

    info!(
        "Connections: {}, Clients: {:?}",
        connections.len(),
        connections
            .iter()
            .map(Arc::strong_count)
            .collect::<Vec<usize>>()
    );

    for channel in connections.iter() {
        if Arc::strong_count(channel) < CLIENTS_PER_CONN {
            return channel.clone();
        }
    }

    let conn = create_client().await;
    let channel = Arc::new(create_event_stream_channel(conn).await);
    connections.push(channel.clone());
    channel
}
// * ---

impl EventV1 {
    /// Publish helper wrapper
    pub async fn p(self, channel: String) {
        #[cfg(debug_assertions)]
        info!("Publishing event to {channel}: {self:?}");

        static QUEUE_NAME: &str = "revolt.events";

        let mut headers: FieldTable = Default::default();
        headers.insert(
            "x-stream-filter-value".into(),
            AMQPValue::LongString(channel.into()),
        );

        let properties: AMQPProperties = Default::default();

        let result = get_event_stream_channel()
            .await
            .basic_publish(
                "",
                QUEUE_NAME,
                Default::default(),
                &rmp_serde::to_vec_named(&self).unwrap(),
                properties.with_headers(headers),
            )
            .await;

        #[cfg(not(debug_assertions))]
        result.ok();

        #[cfg(debug_assertions)]
        result.unwrap();
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
