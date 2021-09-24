use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, EmptyResponse, Result};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn delete_bot(user: User, target: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    let bot = target.fetch_bot().await?;
    if bot.owner != user.id {
        return Err(Error::MissingPermission);
    }

    db_conn().delete_user(&bot.id).await?;
    ClientboundNotification::UserUpdate {
        id: target.id.clone(),
        data: json!({
            "username": format!("Deleted User {}", &bot.id),
            "flags": 2
        }),
        clear: None,
    }
    .publish_as_user(target.id.clone());
    db_conn().delete_bot(&bot.id).await?;

    Ok(EmptyResponse {})
}
