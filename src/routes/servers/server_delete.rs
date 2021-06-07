use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<()> {
    let target = target.fetch_server().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_view() {
        return Err(Error::MissingPermission);
    }

    if user.id == target.owner {
        target.delete().await
    } else {
        target.remove_member(&user.id).await
    }
}
