use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use mongodb::options::UpdateOptions;

#[put("/<target>/ack/<message>")]
pub async fn req(user: User, target: Ref, message: Ref) -> Result<()> {
    let target = target.fetch_channel().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let id = target.id();
    get_collection("channel_unreads")
        .update_one(
            doc! {
                "_id.channel": id,
                "_id.user": &user.id
            },
            doc! {
                "$unset": {
                    "mentions": 1
                },
                "$set": {
                    "last_id": &message.id
                }
            },
            UpdateOptions::builder().upsert(true).build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "channel_unreads",
        })?;

    ClientboundNotification::ChannelAck {
        id: id.to_string(),
        user: user.id.clone(),
        message_id: message.id,
    }
    .publish(user.id);

    Ok(())
}
