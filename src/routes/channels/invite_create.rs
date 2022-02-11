use revolt_quark::{
    models::{Invite, User},
    perms, ChannelPermission, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;

#[post("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Invite>> {
    let channel = target.as_channel(db).await?;

    if !perms(&user)
        .channel(&channel)
        .calc_channel(db)
        .await
        .get_invite_others()
    {
        return Err(Error::MissingPermission {
            permission: ChannelPermission::InviteOthers as i32,
        });
    }

    Invite::create(db, &user, &channel).await.map(Json)
}
