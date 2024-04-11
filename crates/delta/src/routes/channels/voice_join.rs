use std::borrow::Cow;

use revolt_config::config;
use revolt_models::v0;
use revolt_database::{util::{permissions::perms, reference::Reference}, Channel, Database, User};
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};

use livekit_api::{access_token::{AccessToken, VideoGrants}, services::room::{CreateRoomOptions, RoomClient}};
use rocket::{serde::json::Json, State};

/// # Join Call
///
/// Asks the voice server for a token to join the call.
#[openapi(tag = "Voice")]
#[post("/<target>/join_call")]
pub async fn call(db: &State<Database>, rooms: &State<RoomClient>, user: User, target: Reference) -> Result<Json<v0::CreateVoiceUserResponse>> {
    let channel = target.as_channel(db).await?;

    let mut permissions = perms(db, &user).channel(&channel);

    let current_permissions = calculate_channel_permissions(&mut permissions).await;
    current_permissions.throw_if_lacking_channel_permission(ChannelPermission::Connect)?;

    // TODO - decide if we should allow being in multiple voice channels for users

    // if user.current_voice_channel(&channel.server().unwrap_or_else(|| channel.id()))
    //     .await?
    //     .is_some()
    // {
    //     return Err(create_error!(AlreadyInVoiceChannel))
    // }

    let config = config().await;

    if config.api.livekit.url.is_empty() {
        return Err(create_error!(LiveKitUnavailable));
    };

    let voice = match &channel {
        Channel::DirectMessage { .. } | Channel::VoiceChannel { .. } => Cow::Owned(v0::VoiceInformation::default()),
        Channel::TextChannel { voice: Some(voice), .. } => Cow::Borrowed(voice),
        _ => return Err(create_error!(CannotJoinCall))
    };

    let mut allowed_sources = Vec::new();

    if current_permissions.has(ChannelPermission::Speak as u64) {
        allowed_sources.push("MICROPHONE".to_string())
    };

    if current_permissions.has(ChannelPermission::Video as u64) {
        allowed_sources.extend(["CAMERA".to_string(), "SCREEN_SHARE".to_string(), "SCREEN_SHARE_AUDIO".to_string()]);
    };

    let token = AccessToken::with_api_key(&config.api.livekit.key, &config.api.livekit.secret)
        .with_name(&format!("{}#{}", user.username, user.discriminator))
        .with_identity(&user.id)
        .with_metadata(&serde_json::to_string(&user).map_err(|_| create_error!(InternalError))?)
        .with_grants(VideoGrants {
            room_join: true,
            can_publish_sources: allowed_sources,
            room: channel.id().to_string(),
            ..Default::default()
        })
        .to_jwt()
        .inspect_err(|e| log::error!("{e:?}"))
        .map_err(|_| create_error!(InternalError))?;

    let room = rooms.create_room(&channel.id(), CreateRoomOptions {
        max_participants: voice.max_users.unwrap_or(u32::MAX),
        empty_timeout: 5 * 60,  // 5 minutes
        ..Default::default()
    })
    .await
    .inspect_err(|e| log::error!("{e:?}"))
    .map_err(|_| create_error!(InternalError))?;

    log::info!("created room {room:?}");

    Ok(Json(v0::CreateVoiceUserResponse { token }))
}
