use crate::database::*;
use crate::util::result::{Error, Result};

use rocket_contrib::json::JsonValue;

// ! FIXME: this is a temporary route while permissions are being worked on.

#[get("/<target>/members")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;
    
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let members = Server::fetch_members(&target.id).await?;
    Ok(json!(user.fetch_multiple_users(members).await?))
}
