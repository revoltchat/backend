use revolt_quark::models::User;
use revolt_quark::{Database, Error, Ref, Result};

use rocket::serde::json::Json;
use rocket::State;

/// # Accept Friend Request
///
/// Accept another user's friend request.
#[openapi(tag = "Relationships")]
#[put("/<target>/friend")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<User>> {
    let mut target = target.as_user(db).await?;

    if user.bot.is_some() || target.bot.is_some() {
        return Err(Error::IsBot);
    }

    user.add_friend(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
