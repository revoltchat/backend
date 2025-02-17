use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    voice::{delete_voice_state, get_channel_node, get_user_voice_channel_in_server, VoiceClient},
    Database, RemovalIntention, User,
};
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Kick Member
///
/// Removes a member from the server.
#[openapi(tag = "Server Members")]
#[delete("/<target>/members/<member>")]
pub async fn kick(
    db: &State<Database>,
    voice_client: &State<VoiceClient>,
    user: User,
    target: Reference,
    member: Reference,
) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;

    if member.id == user.id {
        return Err(create_error!(CannotRemoveYourself));
    }

    if member.id == server.owner {
        return Err(create_error!(InvalidOperation));
    }

    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::KickMembers)?;

    let member = member.as_member(db, &server.id).await?;
    if member.get_ranking(query.server_ref().as_ref().unwrap())
        <= query.get_member_rank().unwrap_or(i64::MIN)
    {
        return Err(create_error!(NotElevated));
    }

    if let Some(channel_id) = get_user_voice_channel_in_server(&user.id, &server.id).await? {
        let node = get_channel_node(&channel_id).await?;

        voice_client.remove_user(&node, &user.id, &channel_id).await?;
        delete_voice_state(&channel_id, Some(&server.id), &user.id).await?;
    }

    member
        .remove(db, &server, RemovalIntention::Kick, false)
        .await
        .map(|_| EmptyResponse)
}
