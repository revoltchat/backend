use revolt_quark::{Error, Result, models::{User, Channel}, Ref, Database, perms, ChannelPermission};

use rocket::{serde::json::Json, State};

#[get("/<target>")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<Channel>> {
    let target = target.as_channel(db).await?;

    if perms(&user).channel(&target).calc_channel(db).await.get_view() {
        Ok(Json(target))
    } else {
        Err(Error::MissingPermission { permission: ChannelPermission::View as i32 })
    }
}
