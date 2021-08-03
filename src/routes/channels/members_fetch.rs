use crate::database::*;
use crate::util::result::{Error, Result};

use rocket::serde::json::Value;

#[get("/<target>/members")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let target = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    if let Channel::Group { recipients, .. } = target {
        Ok(json!(user.fetch_multiple_users(recipients).await?))
    } else {
        Err(Error::InvalidOperation)
    }
}
