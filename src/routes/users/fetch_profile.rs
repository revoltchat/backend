//! Fetch another user's profile
//! 
//! Will fail if the authenticated user does not
//! have permission to access the other user's profile.

use revolt_quark::{Ref, Result, Error, models::{User, user::UserProfile}, perms, Database};

use rocket::{serde::json::Json, State};

#[get("/<target>/profile")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<UserProfile>> {
    let target = target.as_user(db).await?;

    if perms(&user).user(&target).calc_user(db).await.get_view_profile() {
        Ok(Json(target.profile.unwrap_or_default()))
    } else {
        Err(Error::NotFound)
    }
}
