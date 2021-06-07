use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;

#[delete("/<server>/bans/<target>")]
pub async fn req(user: User, server: Ref, target: Ref) -> Result<()> {    
    let server = server.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&server)
        .for_server()
        .await?;
    
    if !perm.get_ban_members() {
        Err(Error::MissingPermission)?
    }

    if target.id == user.id {
        return Err(Error::InvalidOperation)
    }

    if target.id == server.owner {
        return Err(Error::MissingPermission)
    }

    let target = target.fetch_ban(&server.id).await?;
    get_collection("server_bans")
        .delete_one(
            doc! {
                "_id.server": &server.id,
                "_id.user": &target.id.user
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "delete_one",
            with: "server_ban"
        })?;
    
    Ok(())
}
