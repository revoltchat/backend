use revolt_quark::{
    models::{File, User},
    Db, Error, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

/// # Public Bot
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PublicBot {
    /// Bot Id
    #[serde(rename = "_id")]
    id: String,
    /// Bot Username
    username: String,
    /// Profile Avatar
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<File>,
    /// Profile Description
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// # Fetch Public Bot
///
/// Fetch details of a public (or owned) bot by its id.
#[openapi(tag = "Bots")]
#[get("/<target>/invite")]
pub async fn fetch_public_bot(db: &Db, user: Option<User>, target: Ref) -> Result<Json<PublicBot>> {
    let bot = target.as_bot(db).await?;
    if !bot.public && user.map_or(true, |x| x.id != bot.owner) {
        return Err(Error::NotFound);
    }

    let user = db.fetch_user(&bot.id).await?;

    Ok(Json(PublicBot {
        id: bot.id,
        username: user.username,
        avatar: user.avatar,
        description: user.profile.and_then(|p| p.content),
    }))
}
