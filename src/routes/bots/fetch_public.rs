use revolt_quark::{Db, Error, Ref, Result};

use serde_json::Value;

#[get("/<target>/invite")]
pub async fn fetch_public_bot(db: &Db, target: Ref) -> Result<Value> {
    let bot = target.as_bot(db).await?;
    if !bot.public {
        return Err(Error::NotFound);
    }

    let user = db.fetch_user(&bot.id).await?;

    Ok(json!({
        "_id": bot.id,
        "username": user.username,
        "avatar": user.avatar,
        "description": user.profile.map(|p| p.content)
    }))
}
