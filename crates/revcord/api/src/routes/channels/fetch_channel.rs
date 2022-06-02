use revolt_quark::{perms, Database, Permission, Ref, Result};
use revcord_models::{QuarkConversion, channel::Channel, user::User};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Channel Information")]
#[get("/<target>")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<Channel>> {
    let channel = target.as_channel(db).await?;
    perms(&user.to_quark().await)
        .channel(&channel)
        .throw_permission(db, Permission::ViewChannel)
        .await?;

    Ok(Json(Channel::from_quark(channel).await))
}
