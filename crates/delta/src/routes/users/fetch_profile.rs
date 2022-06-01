use revolt_quark::{
    models::{user::UserProfile, User},
    perms, Database, Error, Ref, Result,
};

use rocket::{serde::json::Json, State};

/// # Fetch User Profile
///
/// Retrieve a user's profile data.
///
/// Will fail if you do not have permission to access the other user's profile.
#[openapi(tag = "User Information")]
#[get("/<target>/profile")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<UserProfile>> {
    let target = target.as_user(db).await?;

    if perms(&user)
        .user(&target)
        .calc_user(db)
        .await
        .get_view_profile()
    {
        Ok(Json(target.profile.unwrap_or_default()))
    } else {
        Err(Error::NotFound)
    }
}
