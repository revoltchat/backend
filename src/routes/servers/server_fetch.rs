use crate::database::*;
use crate::util::result::{Error, Result};

use rocket::serde::json::Value;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    Ok(json!(target))
}
