use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<()> {
    let target = target.fetch_server().await?;

    /*let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }*/

    // ! FIXME: either delete server if owner
    // ! OR leave server if member

    // also need to delete server invites
    // and members
    // and bans

    target.delete().await
}
