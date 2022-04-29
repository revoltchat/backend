use revolt_quark::{models::User, Db, EmptyResponse, Error, Ref, Result};

/// # Delete Bot
///
/// Delete a bot by its id.
#[openapi(tag = "Bots")]
#[delete("/<target>")]
pub async fn delete_bot(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    bot.delete(db).await.map(|_| EmptyResponse)
}
