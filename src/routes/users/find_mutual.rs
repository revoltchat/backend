//! Fetch IDs of servers and friends that are mutually shared between the
//! authenticated user and the target user

use revolt_quark::models::User;
use revolt_quark::{Error, Result, perms, Database, Ref};

use rocket::State;
use rocket::serde::json::Value;

#[get("/<target>/mutual")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Value> {
    let target = target.as_user(db).await?;

    if perms(&user).user(&target).calc_user(db).await.get_view_profile() {
        Ok(json!({
            "users": db.fetch_mutual_user_ids(&user.id, &target.id).await?,
            "servers": db.fetch_mutual_server_ids(&user.id, &target.id).await?
        }))
    } else {
        Err(Error::NotFound)
    }
}
