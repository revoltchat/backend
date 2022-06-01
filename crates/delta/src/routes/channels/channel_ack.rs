use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Permission, Ref, Result};

/// # Acknowledge Message
///
/// Lets the server and all other clients know that we've seen this message id in this channel.
#[openapi(tag = "Messaging")]
#[put("/<target>/ack/<message>")]
pub async fn req(db: &Db, user: User, target: Ref, message: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission(db, Permission::ViewChannel)
        .await?;

    channel
        .ack(&user.id, &message.id)
        .await
        .map(|_| EmptyResponse)
}
