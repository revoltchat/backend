use revolt_quark::{
    models::{Invite, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;

#[post("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Invite>> {
    let channel = target.as_channel(db).await?;

    if !perms(&user)
        .channel(&channel)
        .calc(db)
        .await
        .can_invite_others()
    {
        return Error::from_permission(Permission::InviteOthers);
    }

    Invite::create(db, &user, &channel).await.map(Json)
}
