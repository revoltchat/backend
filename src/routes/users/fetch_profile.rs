use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::Value;

#[get("/<target>/profile")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let target = target.fetch_user().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_user(&target)
        .for_user_given()
        .await?;

    if !perm.get_view_profile() {
        Err(Error::MissingPermission)?
    }

    if target.profile.is_some() {
        Ok(json!(target.profile))
    } else {
        Ok(json!({}))
    }
}
