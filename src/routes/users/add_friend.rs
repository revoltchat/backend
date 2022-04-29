use revolt_quark::models::User;
use revolt_quark::{Database, Result};

use rocket::serde::json::Json;
use rocket::State;

/// # Send Friend Request / Accept Request
///
/// Send a friend request to another user or accept another user's friend request.
#[openapi(tag = "Relationships")]
#[put("/<username>/friend")]
pub async fn req(db: &State<Database>, user: User, username: String) -> Result<Json<User>> {
    let mut target = db.fetch_user_by_username(&username).await?;
    user.add_friend(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
