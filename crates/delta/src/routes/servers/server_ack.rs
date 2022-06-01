use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result};

/// # Mark Server As Read
///
/// Mark all channels in a server as read.
#[openapi(tag = "Server Information")]
#[put("/<target>/ack")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    db.acknowledge_channels(&user.id, &server.channels)
        .await
        .map(|_| EmptyResponse)
}
