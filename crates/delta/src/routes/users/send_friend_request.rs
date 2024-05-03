use revolt_quark::models::User;
use revolt_quark::{Database, Error, Result};

use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

/// # User Lookup Information
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataSendFriendRequest {
    /// Username and discriminator combo separated by #
    username: String,
}

/// # Send Friend Request
///
/// Send a friend request to another user.
#[openapi(tag = "Relationships")]
#[post("/friend", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    data: Json<DataSendFriendRequest>,
) -> Result<Json<User>> {
    if let Some((username, discriminator)) = data.username.split_once('#') {
        let mut target = db.fetch_user_by_username(username, discriminator).await?;

        if user.bot.is_some() || target.bot.is_some() {
            return Err(Error::IsBot);
        }

        user.add_friend(db, &mut target).await?;
        Ok(Json(target.with_auto_perspective(db, &user).await))
    } else {
        Err(Error::InvalidProperty)
    }
}
