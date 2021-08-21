use crate::util::result::{Error, Result, EmptyResponse};
use crate::{database::*, notifications::events::ClientboundNotification};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<EmptyResponse> {

    let server = target.fetch_server().await?;
    let target = target.fetch_channel().await?;
    let serverPerm = permissions::PermissionCalculator::new(&user)
        .with_server(&server)
        .for_channel()
        .await?;
    let channelPerm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !serverPerm.get_manage_channel() && !channelPerm.get_manage_channel() {
        Err(Error::MissingPermission)?
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

            Ok(EmptyResponse {})
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
                    target.delete().await?;
                    return Ok(EmptyResponse {});
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
            .publish(id.clone());

            Content::SystemMessage(SystemMessage::UserLeft { id: user.id })
                .send_as_system(&target)
                .await
                .ok();

            Ok(EmptyResponse {})
        }
        Channel::TextChannel { .. } |
        Channel::VoiceChannel { .. } => {
            if perm.get_manage_channel() {
                target.delete().await?;
                Ok(EmptyResponse {})
            } else {
                Err(Error::MissingPermission)
            }
        }
    }
}
