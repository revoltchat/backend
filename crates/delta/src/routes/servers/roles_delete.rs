use livekit_protocol::ParticipantPermission;
use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    voice::{get_allowed_sources, get_voice_channel_members, get_voice_state, update_voice_state, VoiceClient},
    Channel, Database, User
};
use revolt_models::v0::PartialUserVoiceState;
use revolt_permissions::{calculate_channel_permissions, calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Delete Role
///
/// Delete a server role by its id.
#[openapi(tag = "Server Permissions")]
#[delete("/<target>/roles/<role_id>")]
pub async fn delete(
    db: &State<Database>,
    user: User,
    target: Reference,
    role_id: String,
    voice_client: &State<VoiceClient>
) -> Result<EmptyResponse> {
    let mut server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageRole)?;

    let member_rank = query.get_member_rank().unwrap_or(i64::MIN);

    if let Some(role) = server.roles.remove(&role_id) {
        if role.rank <= member_rank {
            return Err(create_error!(NotElevated));
        }

        for channel_id in &server.channels {
            let channel = Reference::from_unchecked(channel_id.clone()).as_channel(db).await?;

            if matches!(channel, Channel::VoiceChannel { .. }) {
                for member_id in get_voice_channel_members(channel_id).await? {
                    let member = Reference::from_unchecked(member_id).as_member(db, &server.id).await?;

                    if member.roles.contains(&role_id) {
                        let user = Reference::from_unchecked(member.id.user.clone()).as_user(db).await?;
                        let voice_state = get_voice_state(channel_id, Some(&server.id), &user.id).await?.unwrap();

                        let mut query = DatabasePermissionQuery::new(db, &user)
                            .member(&member)
                            .channel(&channel)
                            .server(&server);

                        let permissions = calculate_channel_permissions(&mut query).await;

                        let mut update_event = PartialUserVoiceState {
                            id: Some(user.id.clone()),
                            ..Default::default()
                        };

                        let can_video = permissions.has_channel_permission(ChannelPermission::Video);
                        let can_speak = permissions.has_channel_permission(ChannelPermission::Speak);
                        let can_listen = permissions.has_channel_permission(ChannelPermission::Listen);

                        update_event.camera = voice_state.camera.then_some(can_video);
                        update_event.screensharing = voice_state.screensharing.then_some(can_video);
                        update_event.is_publishing = voice_state.is_publishing.then_some(can_speak);

                        update_voice_state(channel_id, Some(&server.id), &user.id, &update_event).await?;

                        voice_client.update_permissions(&user, channel_id, ParticipantPermission {
                            can_subscribe: can_listen,
                            can_publish: can_speak,
                            can_publish_data: can_speak,
                            ..Default::default()
                        }).await?;
                    }
                }
            }
        };

        role.delete(db, &server.id, &role_id)
            .await
            .map(|_| EmptyResponse)
    } else {
        Err(create_error!(NotFound))
    }
}
