use revolt_quark::models::User;
use revolt_quark::{Database, Error, Result};

use rocket::serde::json::Json;
use rocket::State;

/// # Deny Friend Request / Remove Friend
///
/// Denies another user's friend request or removes an existing friend.
#[openapi(tag = "Relationships")]
#[delete("/<target>/friend")]
pub async fn req(db: &State<Database>, user: User, target: String) -> Result<Json<User>> {
    let mut target = db.fetch_user(&target).await?;

    if user.bot.is_some() || target.bot.is_some() {
        return Err(Error::IsBot);
    }

    user.remove_friend(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
