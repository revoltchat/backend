use revolt_config::config;
use revolt_models::v0;
use revolt_database::{util::{permissions::perms, reference::Reference}, voice::{delete_voice_state, get_channel_node, get_user_voice_channels, raise_if_in_voice, set_channel_call_started_system_message, VoiceClient}, Channel, Database, SystemMessage, User, AMQP};
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};

use rocket::{serde::json::Json, State};

/// # Join Call
///
/// Asks the voice server for a token to join the call.
#[openapi(tag = "Voice")]
#[post("/<target>/join_call", data="<data>")]
pub async fn call(
    db: &State<Database>,
    amqp: &State<AMQP>,
    voice_client: &State<VoiceClient>,
    user: User,
    target: Reference,
    data: Json<v0::DataJoinCall>
) -> Result<Json<v0::CreateVoiceUserResponse>> {
    if !voice_client.is_enabled() {
        return Err(create_error!(LiveKitUnavailable))
    }

    let v0::DataJoinCall {
        node,
        force_disconnect
    } = data.into_inner();

    if user.bot.is_some() && force_disconnect == Some(true) {
        return Err(create_error!(IsBot))
    }

    let config = config().await;

    let channel = target.as_channel(db).await?;

    let mut permissions = perms(db, &user).channel(&channel);

    let current_permissions = calculate_channel_permissions(&mut permissions).await;
    current_permissions.throw_if_lacking_channel_permission(ChannelPermission::Connect)?;

    let existing_node = get_channel_node(channel.id()).await?;

    let node = existing_node.or(node)
        .ok_or_else(|| create_error!(UnknownNode))?;

    let node_host = config.hosts.livekit.get(&node)
        .ok_or_else(|| create_error!(UnknownNode))?
        .clone();

    if force_disconnect == Some(true) {
        // Finds and disconnects any existing voice connections by the user,
        // should only ever loop once but just to cover our backs.

        for channel_id in get_user_voice_channels(&user.id).await? {
            let node = get_channel_node(&channel_id).await?.unwrap();
            let channel = Reference::from_unchecked(channel_id.clone()).as_channel(db).await?;

            voice_client.remove_user(&node, &user.id, &channel_id).await?;
            delete_voice_state(&channel_id, channel.server(), &user.id).await?;
        }
    } else {
        raise_if_in_voice(&user, channel.id()).await?;
    }

    let token = voice_client.create_token(&node, &user, current_permissions, &channel).await?;
    let room = voice_client.create_room(&node, &channel).await?;

    log::debug!("Created room {}", room.name);

    let mut call_started_message = SystemMessage::CallStarted {
        by: user.id.clone(),
        finished_at: None
    }
    .into_message(channel.id().to_string());

    set_channel_call_started_system_message(channel.id(), &call_started_message.id).await?;

    call_started_message.send(
        db,
        Some(amqp),
        v0::MessageAuthor::System {
            username: &user.username,
            avatar: user.avatar.as_ref().map(|file| file.id.as_ref()),
        },
        None,
        None,
        &channel, false
    ).await?;

    Ok(Json(v0::CreateVoiceUserResponse { token, url: node_host.clone() }))
}
