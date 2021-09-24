use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{Document, doc, from_document};
use serde_json::Value;

#[get("/@me")]
pub async fn fetch_owned_bots(user: User) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    let bots = db_conn().get_bots_owned_by_user_id(&user.id).await?;
    let users = db_conn().
        get_bot_users_owned_by_user_id(&user.id).await?;
    Ok(json!({
        "bots": bots,
        "users": users
    }))
}
