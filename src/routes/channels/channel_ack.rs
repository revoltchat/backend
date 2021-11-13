use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result, EmptyResponse};

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

    crate::task_queue::task_ack::queue(
        target.id().into(),
        user.id.clone(),
        crate::task_queue::task_ack::AckEvent::AckMessage {
            id: message.id.clone()
        }
    ).await;

    ClientboundNotification::ChannelAck {
        id: target.id().into(),
        user: user.id.clone(),
        message_id: message.id,
    }
    .publish(user.id);

    Ok(EmptyResponse {})
}
