use crate::{QuarkConversion, to_snowflake, to_ulid};
use revolt_quark::models::{Channel as RevoltChannel};
use twilight_model::channel::{Channel as DiscordChannel, ChannelType};
use serde::{Serialize, Deserialize};
use rocket_okapi::{JsonSchema, okapi::schemars::schema::{Schema, SchemaObject}};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Channel(DiscordChannel);

#[async_trait]
impl QuarkConversion for Channel {
    type Type = RevoltChannel;

    async fn to_quark(self) -> Self::Type {
        let DiscordChannel { guild_id, id, kind, last_message_id, name, nsfw, recipients, topic, owner_id, .. } = self.0;

        match kind {
            ChannelType::GuildText => RevoltChannel::TextChannel {
                id: to_ulid(id),
                server: to_ulid(guild_id.unwrap()),
                name: name.unwrap(),
                description: topic,
                icon: None,
                last_message_id: last_message_id.map(to_ulid),
                default_permissions: None,  // TODO
                role_permissions: HashMap::new(),
                nsfw: nsfw.unwrap_or_default()
            },
            ChannelType::Private => RevoltChannel::DirectMessage {
                id: to_ulid(id),
                active: false,
                recipients: recipients.unwrap_or_default().iter().map(|user| to_ulid(user.id)).collect(),
                last_message_id: last_message_id.map(to_ulid)
            },
            ChannelType::GuildVoice => RevoltChannel::VoiceChannel {
                id: to_ulid(id),
                server: to_ulid(guild_id.unwrap()),
                name: name.unwrap(),
                description: topic,
                icon: None,
                default_permissions: None,  // TODO
                role_permissions: HashMap::new(),
                nsfw: nsfw.unwrap_or_default()
            },
            ChannelType::Group => RevoltChannel::Group {
                id: to_ulid(id),
                name: name.unwrap(),
                owner: to_ulid(owner_id.unwrap()),
                description: topic,
                recipients: recipients.unwrap_or_default().iter().map(|user| to_ulid(user.id)).collect(),
                icon: None,
                last_message_id: last_message_id.map(to_ulid),
                permissions: None,  // TODO
                nsfw: nsfw.unwrap_or_default()
            },
            _ => todo!()
        }
    }

    async fn from_quark(data: Self::Type) -> Self {
        Self(DiscordChannel {
            application_id: None,
            bitrate: None,
            default_auto_archive_duration: None,
            guild_id: match &data {
                RevoltChannel::TextChannel { server, ..} => Some(to_snowflake(server.clone())),
                RevoltChannel::VoiceChannel { server, .. } => Some(to_snowflake(server.clone())),
                _ => None
            },
            icon: None,  // TODO,
            id: to_snowflake(data.id()),
            invitable: None,
            kind: match &data {
                RevoltChannel::SavedMessages { .. } => ChannelType::Private,
                RevoltChannel::DirectMessage { .. } => ChannelType::Private,
                RevoltChannel::Group { .. } => ChannelType::Group,
                RevoltChannel::TextChannel { .. } => ChannelType::GuildText,
                RevoltChannel::VoiceChannel { .. } => ChannelType::GuildVoice
            },
            last_message_id: match &data {
                RevoltChannel::Group { last_message_id, .. } => last_message_id.clone().map(to_snowflake),
                RevoltChannel::TextChannel { last_message_id, .. } => last_message_id.clone().map(to_snowflake),
                RevoltChannel::DirectMessage { last_message_id, .. } => last_message_id.clone().map(to_snowflake),
                _ => None
            },
            last_pin_timestamp: None,
            member: None,
            member_count: None,
            message_count: None,
            name: match &data {
                RevoltChannel::Group { name, .. } => Some(name),
                RevoltChannel::TextChannel { name, ..} => Some(name),
                RevoltChannel::VoiceChannel { name, .. } => Some(name),
                _ => None
            }.cloned(),
            newly_created: None,
            nsfw: match &data {
                RevoltChannel::Group { nsfw, .. } => Some(*nsfw),
                RevoltChannel::TextChannel { nsfw, .. } => Some(*nsfw),
                RevoltChannel::VoiceChannel { nsfw, .. } => Some(*nsfw),
                _ => None
            },
            owner_id: None,
            parent_id: None,
            permission_overwrites: None,  // TODO,
            position: None,  // TODO
            rate_limit_per_user: None,
            recipients: None,
            rtc_region: None,
            thread_metadata: None,
            topic: match &data {
                RevoltChannel::Group { description, .. } => description,
                RevoltChannel::TextChannel { description, ..} => description,
                RevoltChannel::VoiceChannel { description, ..} => description,
                _ => &None
            }.clone(),
            user_limit: None,
            video_quality_mode: None
        })
    }
}

impl JsonSchema for Channel {
    fn schema_name() -> String {
        "Channel".to_string()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        Schema::Object(SchemaObject::default())
    }
}

