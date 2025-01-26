use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::{CreateRoomOptions, RoomClient, UpdateParticipantOptions},
};
use livekit_protocol::{ParticipantInfo, ParticipantPermission, Room};
use redis_kiss::{get_connection, redis::Pipeline, AsyncCommands};
use revolt_config::config;
use revolt_database::{Channel, User};
use revolt_models::v0::{self, PartialUserVoiceState, UserVoiceState};
use revolt_permissions::{ChannelPermission, PermissionValue};
use revolt_result::{create_error, Result, ToRevoltError};
use std::{collections::HashMap, time::Duration};

pub async fn raise_if_in_voice(user: &User, target: &str) -> Result<()> {
    let mut conn = get_connection().await.to_internal_error()?;

    if user.bot.is_some()
    // bots can be in as many voice channels as it wants so we just check if its already connected to the one its trying to connect to
        && conn.sismember(format!("vc-{}", &user.id), target)
            .await
            .to_internal_error()?
    {
        Err(create_error!(AlreadyConnected))
    } else if conn
        .scard::<_, u32>(format!("vc-{}", &user.id)) // check if the current vc set is empty
        .await
        .to_internal_error()?
        > 0
    {
        Err(create_error!(AlreadyInVoiceChannel))
    } else {
        Ok(())
    }
}

pub async fn get_user_voice_channel_in_server(
    user_id: &str,
    server_id: &str,
) -> Result<Option<String>> {
    let mut conn = get_connection().await.to_internal_error()?;

    let unique_key = format!("{}-{}", user_id, server_id);

    conn.get::<&str, Option<String>>(&unique_key)
        .await
        .to_internal_error()
}

pub fn get_allowed_sources(permissions: PermissionValue) -> Vec<&'static str> {
    let mut allowed_sources = Vec::new();

    if permissions.has(ChannelPermission::Speak as u64) {
        allowed_sources.push("MICROPHONE")
    };

    if permissions.has(ChannelPermission::Video as u64) {
        allowed_sources.extend(["CAMERA", "SCREEN_SHARE", "SCREEN_SHARE_AUDIO"]);
    };

    allowed_sources
}

pub async fn create_voice_state(
    channel_id: &str,
    server_id: Option<&str>,
    user_id: &str,
) -> Result<UserVoiceState> {
    let unique_key = format!("{}-{}", &user_id, server_id.unwrap_or(channel_id));

    let voice_state = UserVoiceState {
        id: user_id.to_string(),
        is_receiving: true,
        is_publishing: false,
        screensharing: false,
        camera: false,
    };

    Pipeline::new()
        .sadd(format!("vc-members-{channel_id}"), user_id)
        .sadd(format!("vc-{user_id}"), channel_id)
        .set(&unique_key, channel_id)
        .set(
            format!("is_publishing-{unique_key}"),
            voice_state.is_publishing,
        )
        .set(
            format!("is_receiving-{unique_key}"),
            voice_state.is_receiving,
        )
        .set(
            format!("screensharing-{unique_key}"),
            voice_state.screensharing,
        )
        .set(format!("camera-{unique_key}"), voice_state.camera)
        .query_async(&mut get_connection().await.to_internal_error()?.into_inner())
        .await
        .to_internal_error()?;

    Ok(voice_state)
}

pub async fn delete_voice_state(
    channel_id: &str,
    server_id: Option<&str>,
    user_id: &str,
) -> Result<()> {
    let unique_key = format!("{}-{}", &user_id, server_id.unwrap_or(channel_id));

    Pipeline::new()
        .srem(format!("vc-members-{channel_id}"), user_id)
        .srem(format!("vc-{user_id}"), channel_id)
        .del(&[
            format!("is_publishing-{unique_key}"),
            format!("is_receiving-{unique_key}"),
            format!("screensharing-{unique_key}"),
            format!("camera-{unique_key}"),
            unique_key.clone(),
        ])
        .query_async(&mut get_connection().await.to_internal_error()?.into_inner())
        .await
        .to_internal_error()?;

    Ok(())
}

pub async fn update_voice_state_tracks(
    channel_id: &str,
    server_id: Option<&str>,
    user_id: &str,
    added: bool,
    track: i32,
) -> Result<PartialUserVoiceState> {
    let partial = match track {
        /* TrackSource::Unknown */ 0 => PartialUserVoiceState::default(),
        /* TrackSource::Camera */
        1 => PartialUserVoiceState {
            camera: Some(added),
            ..Default::default()
        },
        /* TrackSource::Microphone */
        2 => PartialUserVoiceState {
            is_publishing: Some(added),
            ..Default::default()
        },
        /* TrackSource::ScreenShare | TrackSource::ScreenShareAudio */
        3 | 4 => PartialUserVoiceState {
            screensharing: Some(added),
            ..Default::default()
        },
        _ => unreachable!(),
    };

    update_voice_state(channel_id, server_id, user_id, &partial).await?;

    Ok(partial)
}

pub async fn update_voice_state(
    channel_id: &str,
    server_id: Option<&str>,
    user_id: &str,
    partial: &PartialUserVoiceState,
) -> Result<()> {
    let unique_key = format!("{}-{}", &user_id, server_id.unwrap_or(channel_id));

    let mut pipeline = Pipeline::new();

    if let Some(camera) = &partial.camera {
        pipeline.set(format!("camera-{unique_key}"), camera);
    };

    if let Some(is_publishing) = &partial.is_publishing {
        pipeline.set(format!("is_publishing-{unique_key}"), is_publishing);
    }

    if let Some(is_receiving) = &partial.is_receiving {
        pipeline.set(format!("is_receiving-{unique_key}"), is_receiving);
    }

    if let Some(screensharing) = &partial.screensharing {
        pipeline.set(format!("screensharing-{unique_key}"), screensharing);
    }

    pipeline
        .query_async(&mut get_connection().await.to_internal_error()?.into_inner())
        .await
        .to_internal_error()?;

    Ok(())
}

pub async fn get_voice_channel_members(channel_id: &str) -> Result<Vec<String>> {
    get_connection()
        .await
        .to_internal_error()?
        .smembers::<_, Vec<String>>(format!("vc-members-{}", channel_id))
        .await
        .to_internal_error()
}

pub async fn get_voice_state(
    channel_id: &str,
    server_id: Option<&str>,
    user_id: &str,
) -> Result<Option<UserVoiceState>> {
    let unique_key = format!("{}-{user_id}", server_id.unwrap_or(channel_id));

    let (is_publishing, is_receiving, screensharing, camera) = get_connection()
        .await
        .to_internal_error()?
        .mget::<_, (Option<bool>, Option<bool>, Option<bool>, Option<bool>)>(&[
            format!("is_publishing-{unique_key}"),
            format!("is_receiving-{unique_key}"),
            format!("screensharing-{unique_key}"),
            format!("camera-{unique_key}"),
        ])
        .await
        .to_internal_error()?;

    match (is_publishing, is_receiving, screensharing, camera) {
        (Some(is_publishing), Some(is_receiving), Some(screensharing), Some(camera)) => {
            Ok(Some(v0::UserVoiceState {
                id: user_id.to_string(),
                is_receiving,
                is_publishing,
                screensharing,
                camera,
            }))
        }
        _ => Ok(None),
    }
}

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
                can_publish_sources: allowed_sources.into_iter().map(ToString::to_string).collect(),
                can_subscribe: permissions.has_channel_permission(ChannelPermission::Listen),
                room: channel.id().to_string(),
                ..Default::default()
            })
            .to_jwt()
            .to_internal_error()
    }

    pub async fn create_room(&self, channel: &Channel) -> Result<Room> {
        let voice = channel
            .voice()
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
}
