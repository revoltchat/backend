use crate::util::result::{Error, Result, EmptyResponse};
use crate::{database::*, notifications::events::ClientboundNotification};

use mongodb::bson::doc;

#[delete("/<target>/recipients/<member>")]
pub async fn req(user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    if &user.id == &member.id {
        Err(Error::CannotRemoveYourself)?
    }

    let channel = target.fetch_channel().await?;

    if let Channel::Group {
        id,
        owner,
        recipients,
        ..
    } = &channel
    {
        if &user.id != owner {
            // figure out if we want to use perm system here
            Err(Error::MissingPermission)?
        }

        if recipients.iter().find(|x| *x == &member.id).is_none() {
            Err(Error::NotInGroup)?
        }

        get_collection("channels")
            .update_one(
                doc! {
                    "_id": &id
                },
                doc! {
                    "$pull": {
                        "recipients": &member.id
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })?;

        ClientboundNotification::ChannelGroupLeave {
            id: id.clone(),
            user: member.id.clone(),
        }
        .publish(id.clone());

        Content::SystemMessage(SystemMessage::UserRemove {
            id: member.id,
            by: user.id,
        })
        .send_as_system(&channel)
        .await
        .ok();

        Ok(EmptyResponse {})
    } else {
        Err(Error::InvalidOperation)
    }
}
