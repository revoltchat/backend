use crate::util::result::{Error, Result};
use crate::util::variables::MAX_GROUP_SIZE;
use crate::{database::*, notifications::events::ClientboundNotification};

use mongodb::bson::doc;

#[put("/<target>/recipients/<member>")]
pub async fn req(user: User, target: Ref, member: Ref) -> Result<()> {
    if get_relationship(&user, &member.id) != RelationshipStatus::Friend {
        Err(Error::NotFriends)?
    }

    let channel = target.fetch_channel().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    if let Channel::Group { id, recipients, .. } = &channel {
        if recipients.len() >= *MAX_GROUP_SIZE {
            Err(Error::GroupTooLarge {
                max: *MAX_GROUP_SIZE,
            })?
        }

        if recipients.iter().find(|x| *x == &member.id).is_some() {
            Err(Error::AlreadyInGroup)?
        }

        get_collection("channels")
            .update_one(
                doc! {
                    "_id": &id
                },
                doc! {
                    "$push": {
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

        ClientboundNotification::ChannelGroupJoin {
            id: id.clone(),
            user: member.id.clone(),
        }
        .publish(id.clone())
        .await
        .ok();

        Message::create(
            "00000000000000000000000000".to_string(),
            id.clone(),
            format!("<@{}> added <@{}> to the group.", user.id, member.id),
        )
        .publish(&channel)
        .await
        .ok();

        Ok(())
    } else {
        Err(Error::InvalidOperation)
    }
}
