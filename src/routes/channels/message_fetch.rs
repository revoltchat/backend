use revolt_quark::{
    models::{Message, User},
    perms, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;

#[get("/<target>/messages/<msg>")]
pub async fn req(db: &Db, user: User, target: Ref, msg: Ref) -> Result<Json<Message>> {
    let channel = target.as_channel(db).await?;
    if !perms(&user)
        .channel(&channel)
        .calc(db)
        .await
        .can_view_channel()
    {
        return Err(Error::NotFound);
    }

    let message = msg.as_message(db).await?;
    if message.channel != channel.as_id() {
        return Err(Error::NotFound);
    }

    Ok(Json(message))
}
