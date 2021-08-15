use crate::database::*;
use crate::util::result::{Error, Result};

use rocket::serde::json::Value;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let mut target = target.fetch_user().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_user(&target)
        .for_user_given()
        .await?;

    if !perm.get_access() {
        Err(Error::MissingPermission)?
    }

    Ok(json!(target.from(&user).with(perm)))
}
