use crate::database::*;
use crate::util::result::{Error, Result};

use serde_json::Value;

#[get("/<target>")]
pub async fn fetch_bot(user: User, target: Ref) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    let bot = target.fetch_bot().await?;

    if !bot.public {
        if bot.owner != user.id {
            return Err(Error::BotIsPrivate);
        }
    }

    let user = Ref::from_unchecked(bot.id.clone()).fetch_user().await?;
    
    Ok(json!({
        "bot": bot,
        "user": user
    }))
}
