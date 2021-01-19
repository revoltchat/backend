use crate::util::result::{Error, Result};
use crate::{database::*, notifications::events::ClientboundNotification};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<()> {
    let target = target.fetch_channel().await?;

    let perm = permissions::channel::calculate(&user, &target).await;
    if !perm.get_view() {
        Err(Error::LabelMe)?
    }

    match &target {
        Channel::SavedMessages { .. } => Err(Error::NoEffect),
        Channel::DirectMessage { .. } => {
            get_collection("channels")
                .update_one(
                    doc! {
                        "_id": target.id()
                    },
                    doc! {
                        "$set": {
                            "active": false
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "channel",
                })?;

            Ok(())
        }
        Channel::Group {
            id,
            owner,
            recipients,
            ..
        } => {
            if &user.id == owner {
                if let Some(new_owner) = recipients.iter().find(|x| *x != &user.id) {
                    get_collection("channels")
                        .update_one(
                            doc! {
                                "_id": &id
                            },
                            doc! {
                                "$set": {
                                    "owner": new_owner
                                },
                                "$pull": {
                                    "recipients": &user.id
                                }
                            },
                            None,
                        )
                        .await
                        .map_err(|_| Error::DatabaseError {
                            operation: "update_one",
                            with: "channel",
                        })?;

                    target.publish_update(json!({ "owner": new_owner })).await?;
                } else {
                    return target.delete().await;
                }
            } else {
                get_collection("channels")
                    .update_one(
                        doc! {
                            "_id": &id
                        },
                        doc! {
                            "$pull": {
                                "recipients": &user.id
                            }
                        },
                        None,
                    )
                    .await
                    .map_err(|_| Error::DatabaseError {
                        operation: "update_one",
                        with: "channel",
                    })?;
            }

            ClientboundNotification::ChannelGroupLeave {
                id: id.clone(),
                user: user.id.clone(),
            }
            .publish(id.clone())
            .await
            .ok();

            Message::create(
                "00000000000000000000000000".to_string(),
                id.clone(),
                format!("<@{}> left the group.", user.id),
            )
            .publish()
            .await
            .ok();

            Ok(())
        }
    }
}
