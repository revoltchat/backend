use revolt_quark::{models::User, perms, Database, Error, Ref, Result};

use rocket::{serde::json::Json, State};

/// # Fetch User
///
/// Retrieve a user's information.
#[openapi(tag = "User Information")]
#[get("/<target>")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<User>> {
    let target = target.as_user(db).await?;

    let permissions = perms(&user).user(&target).calc_user(db).await;
    if permissions.get_access() {
        Ok(Json(target.with_perspective(&user, &permissions)))
    } else {
        Err(Error::NotFound)
    }
}
