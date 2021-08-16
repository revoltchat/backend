use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, EmptyResponse, Result};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn delete_bot(user: User, target: Ref) -> Result<EmptyResponse> {
    let bot = target.fetch_bot().await?;
    if bot.owner != user.id {
        return Err(Error::MissingPermission);
    }

    let username = format!("Deleted User {}", &bot.id);
    get_collection("users")
        .update_one(
            doc! {
                "_id": &bot.id
            },
            doc! {
                "$set": {
                    "username": &username,
                    "flags": 2
                },
                "$unset": {
                    "avatar": 1,
                    "status": 1,
                    "profile": 1
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            with: "user",
            operation: "update_one"
        })?;

    ClientboundNotification::UserUpdate {
        id: target.id.clone(),
        data: json!({
            "username": username,
            "flags": 2
        }),
        clear: None,
    }
    .publish_as_user(target.id.clone());

    get_collection("bots")
        .delete_one(
            doc! {
                "_id": &bot.id
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            with: "bot",
            operation: "delete_one"
        })?;

    Ok(EmptyResponse {})
}
