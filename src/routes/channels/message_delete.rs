use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;

#[delete("/<target>/messages/<msg>")]
pub async fn req(user: User, target: Ref, msg: Ref) -> Result<()> {
    let channel = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let message = msg.fetch_message(&channel).await?;
    if message.author != user.id && !perm.get_manage_messages() {
        match channel {
            Channel::SavedMessages { .. } => unreachable!(),
            _ => Err(Error::CannotEditMessage)?,
        }
    }

    message.delete().await
}
