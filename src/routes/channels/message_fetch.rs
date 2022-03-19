use revolt_quark::{
    models::{Message, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;

/// # Fetch Message
///
/// Retrieves a message by its id.
#[openapi(tag = "Messaging")]
#[get("/<target>/messages/<msg>")]
pub async fn req(db: &Db, user: User, target: Ref, msg: Ref) -> Result<Json<Message>> {
    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission(db, Permission::ViewChannel)
        .await?;

    let message = msg.as_message(db).await?;
    if message.channel != channel.as_id() {
        return Err(Error::NotFound);
    }

    Ok(Json(message))
}
