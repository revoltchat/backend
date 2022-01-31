//! Fetch another user
//! 
//! Will fail if the authenticated user does not
//! have permission to access the other user.

use revolt_quark::{Ref, Result, Error, models::User, perms, Database};

use rocket::{serde::json::Json, State};

#[get("/<target>")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<User>> {
    let target = target.as_user(db).await?;

    if perms(&user).user(&target).calc_user(db).await.get_access() {
        Ok(Json(target))
    } else {
        Err(Error::NotFound)
    }
}
