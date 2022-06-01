use revolt_quark::{
    models::{Channel, User},
    Db, EmptyResponse, Error, Permission, Ref, Result,
};

/// # Remove Member from Group
///
/// Removes a user from the group.
#[openapi(tag = "Groups")]
#[delete("/<target>/recipients/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;

    match &channel {
        Channel::Group {
            owner, recipients, ..
        } => {
            if &user.id != owner {
                return Error::from_permission(Permission::ManageChannel);
            }

            let member = member.as_user(db).await?;
            if user.id == member.id {
                return Err(Error::CannotRemoveYourself);
            }

            if !recipients.iter().any(|x| *x == member.id) {
                return Err(Error::NotInGroup);
            }

            channel
                .remove_user_from_group(db, &member.id, Some(&user.id))
                .await
                .map(|_| EmptyResponse)
        }
        _ => Err(Error::InvalidOperation),
    }
}
