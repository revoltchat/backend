use livekit_protocol::ParticipantPermission;
use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference}, Channel, Database, PartialRole, User
};
use revolt_models::v0::{self, PartialUserVoiceState};
use revolt_permissions::{calculate_channel_permissions, calculate_server_permissions, ChannelPermission, PermissionQuery};
use revolt_result::{create_error, Result};
use revolt_voice::{get_allowed_sources, get_voice_channel_members, get_voice_state, update_voice_state, update_voice_state_tracks, VoiceClient};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Edit Role
///
/// Edit a role by its id.
#[openapi(tag = "Server Permissions")]
#[patch("/<target>/roles/<role_id>", data = "<data>")]
pub async fn edit(
    db: &State<Database>,
    voice: &State<VoiceClient>,
    user: User,
    target: Reference,
    role_id: String,
    data: Json<v0::DataEditRole>,
    voice_client: &State<VoiceClient>
) -> Result<Json<v0::Role>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let mut server = target.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageRole)?;

    let member_rank = query.get_member_rank().unwrap_or(i64::MIN);

    if let Some(mut role) = server.roles.remove(&role_id) {
        // Prevent us from editing roles above us
        if role.rank <= member_rank {
            return Err(create_error!(NotElevated));
        }

        let v0::DataEditRole {
            name,
            colour,
            hoist,
            rank,
            remove,
        } = data;

        // Prevent us from moving a role above other roles
        if let Some(rank) = &rank {
            if rank <= &member_rank {
                return Err(create_error!(NotElevated));
            }
        }

        let partial = PartialRole {
            name,
            colour,
            hoist,
            rank,
            ..Default::default()
        };

        role.update(
            db,
            &server.id,
            &role_id,
            partial,
            remove
                .map(|v| v.into_iter().map(Into::into).collect())
                .unwrap_or_default(),
        )
        .await?;

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

        Ok(Json(role.into()))
    } else {
        Err(create_error!(NotFound))
    }
}
