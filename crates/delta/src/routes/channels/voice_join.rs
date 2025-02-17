use revolt_models::v0;
use revolt_database::{util::{permissions::perms, reference::Reference}, voice::{raise_if_in_voice, VoiceClient}, Channel, Database, SystemMessage, User, AMQP};
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};

use rocket::{serde::json::Json, State};

/// # Join Call
///
/// Asks the voice server for a token to join the call.
#[openapi(tag = "Voice")]
#[post("/<target>/join_call", data="<data>")]
pub async fn call(db: &State<Database>, amqp: &State<AMQP>, voice: &State<VoiceClient>, user: User, target: Reference, data: Json<v0::DataJoinCall>) -> Result<Json<v0::CreateVoiceUserResponse>> {
    if !voice.rooms.contains_key(&data.node) {
        return Err(create_error!(UnknownNode))
    }

    let channel = target.as_channel(db).await?;

    raise_if_in_voice(&user, channel.id()).await?;

    let mut permissions = perms(db, &user).channel(&channel);

    let current_permissions = calculate_channel_permissions(&mut permissions).await;
    current_permissions.throw_if_lacking_channel_permission(ChannelPermission::Connect)?;

    let token = voice.create_token(&data.node, &user, current_permissions, &channel)?;
    let room = voice.create_room(&data.node, &channel).await?;

    log::debug!("Created room {}", room.name);

    match &channel {
        Channel::DirectMessage { .. } | Channel::Group { .. } => {
            SystemMessage::CallStarted {
                by: user.id.clone()
            }
            .into_message(channel.id().to_string())
            .send(
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
        }
        _ => {}
    };

    Ok(Json(v0::CreateVoiceUserResponse { token }))
}
