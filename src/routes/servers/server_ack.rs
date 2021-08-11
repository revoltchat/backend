use crate::database::*;
use crate::util::result::{Error, Result};

#[put("/<target>/ack")]
pub async fn req(user: User, target: Ref) -> Result<()> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    target.mark_as_read(&user.id).await
}
