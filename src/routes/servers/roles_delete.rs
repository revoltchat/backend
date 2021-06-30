use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;

#[delete("/<target>/roles/<role_id>")]
pub async fn req(user: User, target: Ref, role_id: String) -> Result<()> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_roles() {
        Err(Error::MissingPermission)?
    }

    get_collection("servers")
        .update_one(
            doc! {
                "_id": &role_id
            },
            doc! {
                "$unset": {
                    "roles.".to_owned() + &role_id: 1
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "servers"
        })?;
    
    // remove role from members
    // remove role from channels
    
    ClientboundNotification::ServerRoleDelete {
        id: target.id.clone(),
        role_id
    }
    .publish(target.id);

    Ok(())
}
