use revolt_quark::{
    models::{Invite, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;

/// # Create Invite
///
/// Creates an invite to this channel.
///
/// Channel must be a `TextChannel`.
#[openapi(tag = "Channel Invites")]
#[post("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Invite>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::InviteOthers)
        .await?;

    Invite::create(db, &user, &channel).await.map(Json)
}
