use revolt_quark::{models::File, Db, Error, Ref, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PublicBot {
    #[serde(rename = "_id")]
    id: String,
    username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[get("/<target>/invite")]
pub async fn fetch_public_bot(db: &Db, target: Ref) -> Result<Json<PublicBot>> {
    let bot = target.as_bot(db).await?;
    if !bot.public {
        return Err(Error::NotFound);
    }

    let user = db.fetch_user(&bot.id).await?;

    Ok(Json(PublicBot {
        id: bot.id,
        username: user.username,
        avatar: user.avatar,
        description: user.profile.map(|p| p.content).flatten(),
    }))
}
