use crate::database::*;
use crate::util::result::{Error, Result, EmptyResponse};
use crate::notifications::events::ClientboundNotification;

use mongodb::bson::doc;

#[post("/<target>/messages/<msg>/pin")]
pub async fn req(user: User, target: Ref, msg: Ref) -> Result<EmptyResponse> {
    let channel = target.fetch_channel().await?;
    channel.has_messaging()?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;
    if !perm.get_manage_messages() {
        Err(Error::MissingPermission)?
    }

    let message = msg.fetch_message(&channel).await?;
    if message.pinned {
        Err(Error::AlreadyPinned)?
    };

    let set = doc! { "pinned": true };

    get_collection("messages")
        .update_one(
            doc! {
                "_id": &msg.id
            },
            doc! {
                "$set": set
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "message",
        })?;

    ClientboundNotification::ChannelUnpin {
        user: user.id,
        id: message.channel.clone(),
        message_id: message.id,
    }
    .publish(message.channel);

    Ok(EmptyResponse {})
}
