use crate::database::*;
use crate::util::result::{Error, Result};
use crate::notifications::events::ClientboundNotification;

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
        get_collection("server_members")
            .delete_one(
                doc! {

                },
                None
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "server_member"
            })?;

        ClientboundNotification::ServerMemberLeave {
            id: target.id.clone(),
            user: user.id
        }.publish(target.id);

        Ok(())
    }
}
