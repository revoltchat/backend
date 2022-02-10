use revolt_quark::{
    models::{Bot, User},
    Db, Error, Ref, Result,
};
use rocket::serde::json::Json;

#[get("/<target>")]
pub async fn fetch_bot(db: &Db, user: User, target: Ref) -> Result<Json<Bot>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    Ok(Json(bot))
}
