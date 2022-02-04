use revolt_quark::{
    models::{Channel, User},
    perms, ChannelPermission, Database, Error, Ref, Result,
};

use rocket::{serde::json::Json, State};

#[get("/<target>")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<Channel>> {
    let target = target.as_channel(db).await?;

    if perms(&user)
        .channel(&target)
        .calc_channel(db)
        .await
        .get_view()
    {
        Ok(Json(target))
    } else {
        Err(Error::MissingPermission {
            permission: ChannelPermission::View as i32,
        })
    }
}
