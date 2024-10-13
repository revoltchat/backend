use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference}, Channel, Database, User
};
use revolt_models::v0::PartialUserVoiceState;
use revolt_permissions::{calculate_channel_permissions, calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use revolt_voice::{get_allowed_sources, get_voice_channel_members, get_voice_state};
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

                        let sources = get_allowed_sources(permissions);

                        let mut update_event = PartialUserVoiceState {
                            id: Some(user.id.clone()),
                            ..Default::default()
                        };

                        if !sources.contains(&"CAMERA".to_string()) {
                            update_event.camera = 
                            update_event
                        }

                        if voice_state.camera && !sources.contains(&"MICROPHONE".to_string()) {
                            update_event. = Some(false);
                        }

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
