use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;

#[delete("/<target>/members/<member>")]
pub async fn req(user: User, target: Ref, member: String) -> Result<()> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_members() {
        return Err(Error::MissingPermission);
    }

    let member = Ref::from(member)?.fetch_member(&target.id).await?;
    if member.id.user == user.id {
        return Err(Error::InvalidOperation);
    }

    if target.id == target.owner {
        return Err(Error::MissingPermission);
    }

    target
        .remove_member(&member.id.user, RemoveMember::Kick)
        .await
}
