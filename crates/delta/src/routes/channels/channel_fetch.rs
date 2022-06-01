use revolt_quark::{
    models::{Channel, User},
    perms, Database, Permission, Ref, Result,
};

use rocket::{serde::json::Json, State};

/// # Fetch Channel
///
/// Fetch channel by its id.
#[openapi(tag = "Channel Information")]
#[get("/<target>")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<Channel>> {
    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission(db, Permission::ViewChannel)
        .await?;

    Ok(Json(channel))
}
