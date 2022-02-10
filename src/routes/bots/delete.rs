use rauth::util::EmptyResponse;
use revolt_quark::{models::User, Db, Error, Ref, Result};

#[delete("/<target>")]
pub async fn delete_bot(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    db.delete_bot(&bot.id).await.map(|_| EmptyResponse)
}
