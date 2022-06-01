use revolt_quark::{
    models::{Channel, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
};

/// # Add Member to Group
///
/// Adds another user to the group.
#[openapi(tag = "Groups")]
#[put("/<target>/recipients/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let mut channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::InviteOthers)
        .await?;

    match &channel {
        Channel::Group { .. } => {
            let member = member.as_user(db).await?;
            channel
                .add_user_to_group(db, &member.id, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
        _ => Err(Error::InvalidOperation),
    }
}
