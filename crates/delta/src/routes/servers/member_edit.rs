use std::collections::HashSet;

use futures::TryFutureExt;
use livekit_api::services::room::{RoomClient, UpdateParticipantOptions};
use livekit_protocol::ParticipantPermission;
use redis_kiss::{get_connection, redis::Pipeline, AsyncCommands};
use revolt_database::{
    events::client::EventV1,
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    voice::{get_channel_node, get_user_voice_channel_in_server, move_user, sync_user_voice_permissions, VoiceClient},
    Database, File, PartialMember, User,
};
use revolt_models::v0::{self, FieldsMember, PartialUserVoiceState};

use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result, ToRevoltError};
use rocket::{form::validate::Contains, serde::json::Json, State};
use validator::Validate;

/// # Edit Member
///
/// Edit a member by their id.
#[openapi(tag = "Server Members")]
#[patch("/<server>/members/<target>", data = "<data>")]
pub async fn edit(
    db: &State<Database>,
    voice_client: &State<VoiceClient>,
    user: User,
    server: Reference,
    target: Reference,
    data: Json<v0::DataMemberEdit>,
) -> Result<Json<v0::Member>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    // Fetch server and target member
    let mut server = server.as_server(db).await?;
    let mut member = target.as_member(db, &server.id).await?;
    let target_user = target.as_user(&db).await?;

    // Fetch our currrent permissions
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    let permissions = calculate_server_permissions(&mut query).await;

    // Check permissions in server
    if data.nickname.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&v0::FieldsMember::Nickname))
            .unwrap_or_default()
    {
        if user.id == member.id.user {
            permissions.throw_if_lacking_channel_permission(ChannelPermission::ChangeNickname)?;
        } else {
            permissions.throw_if_lacking_channel_permission(ChannelPermission::ManageNicknames)?;
        }
    }

    if data.avatar.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&v0::FieldsMember::Avatar))
            .unwrap_or_default()
    {
        if user.id == member.id.user {
            permissions.throw_if_lacking_channel_permission(ChannelPermission::ChangeAvatar)?;
        } else {
            return Err(create_error!(InvalidOperation));
        }
    }

    if data.roles.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&v0::FieldsMember::Roles))
            .unwrap_or_default()
    {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::AssignRoles)?;
    }

    if data.timeout.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&v0::FieldsMember::Timeout))
            .unwrap_or_default()
    {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::TimeoutMembers)?;
    }

    if data.can_publish.is_some() {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::MuteMembers)?;
    }

    if data.can_receive.is_some() {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::DeafenMembers)?;
    }

    if let Some(new_channel) = &data.voice_channel {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::MoveMembers)?;

        // ensure the channel we are moving them to is in the server and is a voice channel

        let channel = Reference::from_unchecked(new_channel.clone())
            .as_channel(db)
            .await
            .map_err(|_| create_error!(UnknownChannel))?;

        if !channel.server().is_some_and(|v| v == member.id.server) {
            Err(create_error!(UnknownChannel))?
        }

        if channel.voice().is_none() {
            Err(create_error!(NotAVoiceChannel))?
        }
    }

    // Resolve our ranking
    let our_ranking = query.get_member_rank().unwrap_or(i64::MIN);

    // Check that we have permissions to act against this member
    if member.id.user != user.id
        && member.get_ranking(query.server_ref().as_ref().unwrap()) <= our_ranking
    {
        return Err(create_error!(NotElevated));
    }

    // Check permissions against roles in diff
    if let Some(roles) = &data.roles {
        let current_roles = member.roles.iter().collect::<HashSet<&String>>();

        let new_roles = roles.iter().collect::<HashSet<&String>>();
        let added_roles: Vec<&&String> = new_roles.difference(&current_roles).collect();

        for role_id in added_roles {
            if let Some(role) = server.roles.remove(*role_id) {
                if role.rank <= our_ranking {
                    return Err(create_error!(NotElevated));
                }
            } else {
                return Err(create_error!(InvalidRole));
            }
        }
    }

    // Apply edits to the member object
    let v0::DataMemberEdit {
        nickname,
        avatar,
        roles,
        timeout,
        remove,
        can_publish,
        can_receive,
        voice_channel,
    } = data;

    let mut partial = PartialMember {
        nickname,
        roles,
        timeout,
        can_publish,
        can_receive,
        ..Default::default()
    };

    // 1. Remove fields from object
    if let Some(fields) = &remove {
        if fields.contains(&v0::FieldsMember::Avatar) {
            if let Some(avatar) = &member.avatar {
                db.mark_attachment_as_deleted(&avatar.id).await?;
            }
        }
    }

    // 2. Apply new avatar
    if let Some(avatar) = avatar {
        partial.avatar = Some(File::use_user_avatar(db, &avatar, &user.id, &user.id).await?);
    }

    // TODO: impelement moving users

    let remove_contains_voice = remove
        .as_ref()
        .map(|r| r.contains(FieldsMember::CanPublish) || r.contains(FieldsMember::CanReceive))
        .unwrap_or_default();

    if can_publish.is_some()
        || can_receive.is_some()
        || voice_channel.is_some()
        || remove_contains_voice
    {
        if let Some(channel) = get_user_voice_channel_in_server(&target_user.id, &server.id).await? {
            let node = get_channel_node(&channel).await?;
            let channel = Reference::from_unchecked(channel).as_channel(db).await?;


            sync_user_voice_permissions(db, voice_client, &node, &user, &channel, Some(&server), None).await?;
        };
    };

    member
        .update(
            db,
            partial,
            remove
                .map(|v| v.into_iter().map(Into::into).collect())
                .unwrap_or_default(),
        )
        .await?;

    Ok(Json(member.into()))
}
