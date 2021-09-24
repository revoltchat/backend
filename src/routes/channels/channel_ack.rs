use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result, EmptyResponse};

use mongodb::bson::doc;
use mongodb::options::UpdateOptions;

#[put("/<target>/ack/<message>")]
pub async fn req(user: User, target: Ref, message: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }

    let target = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let id = target.id();
    db_conn()
        .update_last_message_in_channel_unreads(&id, &user.id, &message.id)
        .await?;

    ClientboundNotification::ChannelAck {
        id: id.to_string(),
        user: user.id.clone(),
        message_id: message.id,
    }
    .publish(user.id);

    Ok(EmptyResponse {})
}
