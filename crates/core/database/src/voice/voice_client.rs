use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient as InnerRoomClient, UpdateParticipantOptions},
};
use livekit_protocol::{ParticipantInfo, ParticipantPermission, Room};
use revolt_config::{config, LiveKitNode};
use crate::models::{Channel, User};
use revolt_models::v0;
use revolt_permissions::{ChannelPermission, PermissionValue};
use revolt_result::{create_error, Result, ToRevoltError};
use std::{borrow::Cow, collections::HashMap, time::Duration};

use super::get_allowed_sources;


#[derive(Debug)]
pub struct RoomClient {
    pub client: InnerRoomClient,
    pub node: LiveKitNode
}

#[derive(Debug)]
pub struct VoiceClient {
    pub rooms: HashMap<String, RoomClient>
}

impl VoiceClient {
    pub fn new(nodes: HashMap<String, LiveKitNode>) -> Self {
        Self {
            rooms: nodes
                .into_iter()
                .map(|(name, node)|
                    (
                        name,
                        RoomClient {
                            client: InnerRoomClient::with_api_key(&node.url, &node.key, &node.secret),
                            node
                        }
                    )
                )
                .collect()
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.rooms.is_empty()
    }

    pub async fn from_revolt_config() -> Self {
        let config = config().await;

        Self::new(config.api.livekit.nodes.clone())
    }

    pub fn get_node(&self, name: &str) -> Result<&RoomClient> {
        self.rooms.get(name)
            .ok_or_else(|| create_error!(UnknownNode))
    }

    pub fn create_token(
        &self,
        node: &str,
        user: &User,
        permissions: PermissionValue,
        channel: &Channel,
    ) -> Result<String> {
        let room = self.get_node(node)?;

        let allowed_sources = get_allowed_sources(permissions);

        AccessToken::with_api_key(&room.node.key, &room.node.secret)
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

    pub async fn create_room(&self, node: &str, channel: &Channel) -> Result<Room> {
        let room = self.get_node(node)?;

        let voice = match channel {
            Channel::DirectMessage { .. } | Channel::VoiceChannel { .. } => Some(Cow::Owned(v0::VoiceInformation::default())),
            Channel::TextChannel { voice: Some(voice), .. } => Some(Cow::Borrowed(voice)),
            _ => None
        }
        .ok_or_else(|| create_error!(NotAVoiceChannel))?;

        room.client
            .create_room(
                channel.id(),
                CreateRoomOptions {
                    max_participants: voice.max_users.unwrap_or(u32::MAX),
                    empty_timeout: 5 * 60, // 5 minutes,
                    ..Default::default()
                },
            )
            .await
            .to_internal_error()
    }

    pub async fn update_permissions(
        &self,
        node: &str,
        user: &User,
        channel_id: &str,
        new_permissions: ParticipantPermission,
    ) -> Result<ParticipantInfo> {
        let room = self.get_node(node)?;

        room.client
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

    pub async fn remove_user(&self, node: &str, user_id: &str, channel_id: &str) -> Result<()> {
        let room = self.get_node(node)?;

        room.client.remove_participant(channel_id, user_id)
            .await
            .to_internal_error()
    }
}
