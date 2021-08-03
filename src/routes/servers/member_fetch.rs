use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::Value;

#[get("/<target>/members/<member>")]
pub async fn req(user: User, target: Ref, member: String) -> Result<Value> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    Ok(json!(Ref::from(member)?.fetch_member(&target.id).await?))
}
