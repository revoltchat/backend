use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient, UpdateParticipantOptions},
};
use livekit_protocol::{ParticipantInfo, ParticipantPermission, Room};
use revolt_config::config;
use crate::models::{Channel, User};
use revolt_models::v0;
use revolt_permissions::{ChannelPermission, PermissionValue};
use revolt_result::{create_error, Result, ToRevoltError};
use std::{borrow::Cow, collections::HashMap, time::Duration};

use super::get_allowed_sources;

#[derive(Debug)]
pub struct VoiceClient {
    rooms: RoomClient,
    api_key: String,
    api_secret: String,
}

impl VoiceClient {
    pub fn new(url: String, api_key: String, api_secret: String) -> Self {
        Self {
            rooms: RoomClient::with_api_key(&url, &api_key, &api_secret),
            api_key,
            api_secret,
        }
    }

    pub async fn from_revolt_config() -> Self {
        let config = config().await;

        Self::new(
            config.hosts.livekit,
            config.api.livekit.key,
            config.api.livekit.secret,
        )
    }

    pub fn create_token(
        &self,
        user: &User,
        permissions: PermissionValue,
        channel: &Channel,
    ) -> Result<String> {
        let allowed_sources = get_allowed_sources(permissions);

        AccessToken::with_api_key(&self.api_key, &self.api_secret)
            .with_name(&format!("{}#{}", user.username, user.discriminator))
            .with_identity(&user.id)
            .with_metadata(&serde_json::to_string(&user).to_internal_error()?)
            .with_ttl(Duration::from_secs(10))
            .with_grants(VideoGrants {
                room_join: true,
                can_publish: true,
                can_publish_sources: allowed_sources.into_iter().map(ToString::to_string).collect(),
                can_subscribe: permissions.has_channel_permission(ChannelPermission::Listen),
                room: channel.id().to_string(),
                ..Default::default()
            })
            .to_jwt()
            .to_internal_error()
    }

    pub async fn create_room(&self, channel: &Channel) -> Result<Room> {
        let voice = match channel {
            Channel::DirectMessage { .. } | Channel::VoiceChannel { .. } => Some(Cow::Owned(v0::VoiceInformation::default())),
            Channel::TextChannel { voice: Some(voice), .. } => Some(Cow::Borrowed(voice)),
            _ => None
        }

            .ok_or_else(|| create_error!(NotAVoiceChannel))?;

        self.rooms
            .create_room(
                channel.id(),
                CreateRoomOptions {
                    max_participants: voice.max_users.unwrap_or(u32::MAX),
                    empty_timeout: 5 * 60, // 5 minutes
                    ..Default::default()
                },
            )
            .await
            .to_internal_error()
    }

    pub async fn update_permissions(
        &self,
        user: &User,
        channel_id: &str,
        new_permissions: ParticipantPermission,
    ) -> Result<ParticipantInfo> {
        self.rooms
            .update_participant(
                channel_id,
                &user.id,
                UpdateParticipantOptions {
                    permission: Some(new_permissions),
                    attributes: HashMap::new(),
                    name: "".to_string(),
                    metadata: "".to_string(),
                },
            )
            .await
            .to_internal_error()
    }

    pub async fn remove_user(&self, user_id: &str, channel_id: &str) -> Result<()> {
        self.rooms.remove_participant(channel_id, user_id)
            .await
            .to_internal_error()
    }
}
