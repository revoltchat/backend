use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use futures::try_join;
use mongodb::bson::doc;
use rocket::serde::json::Value;

#[put("/<target>/block")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let col = get_collection("users");

    let target = target.fetch_user().await?;

    match get_relationship(&user, &target.id) {
        RelationshipStatus::User | RelationshipStatus::Blocked => Err(Error::NoEffect),
        RelationshipStatus::BlockedOther => {
            col.update_one(
                doc! {
                    "_id": &user.id,
                    "relations._id": &target.id
                },
                doc! {
                    "$set": {
                        "relations.$.status": "Blocked"
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })?;

            ClientboundNotification::UserRelationship {
                id: user.id.clone(),
                user: target,
                status: RelationshipStatus::Blocked,
            }
            .publish(user.id.clone());

            Ok(json!({ "status": "Blocked" }))
        }
        RelationshipStatus::None => {
            match try_join!(
                col.update_one(
                    doc! {
                        "_id": &user.id
                    },
                    doc! {
                        "$push": {
                            "relations": {
                                "_id": &target.id,
                                "status": "Blocked"
                            }
                        }
                    },
                    None
                ),
                col.update_one(
                    doc! {
                        "_id": &target.id
                    },
                    doc! {
                        "$push": {
                            "relations": {
                                "_id": &user.id,
                                "status": "BlockedOther"
                            }
                        }
                    },
                    None
                )
            ) {
                Ok(_) => {
                    let target = target
                        .from_override(&user, RelationshipStatus::Blocked)
                        .await?;
                    let user = user
                        .from_override(&target, RelationshipStatus::BlockedOther)
                        .await?;
                    let target_id = target.id.clone();

                    ClientboundNotification::UserRelationship {
                        id: user.id.clone(),
                        user: target,
                        status: RelationshipStatus::Blocked,
                    }
                    .publish(user.id.clone());

                    ClientboundNotification::UserRelationship {
                        id: target_id.clone(),
                        user,
                        status: RelationshipStatus::BlockedOther,
                    }
                    .publish(target_id);

                    Ok(json!({ "status": "Blocked" }))
                }
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
        }
        RelationshipStatus::Friend
        | RelationshipStatus::Incoming
        | RelationshipStatus::Outgoing => {
            match try_join!(
                col.update_one(
                    doc! {
                        "_id": &user.id,
                        "relations._id": &target.id
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": "Blocked"
                        }
                    },
                    None
                ),
                col.update_one(
                    doc! {
                        "_id": &target.id,
                        "relations._id": &user.id
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": "BlockedOther"
                        }
                    },
                    None
                )
            ) {
                Ok(_) => {
                    let target = target
                        .from_override(&user, RelationshipStatus::Blocked)
                        .await?;
                    let user = user
                        .from_override(&target, RelationshipStatus::BlockedOther)
                        .await?;
                    let target_id = target.id.clone();

                    ClientboundNotification::UserRelationship {
                        id: user.id.clone(),
                        user: target,
                        status: RelationshipStatus::Blocked,
                    }
                    .publish(user.id.clone());

                    ClientboundNotification::UserRelationship {
                        id: target_id.clone(),
                        user,
                        status: RelationshipStatus::BlockedOther,
                    }
                    .publish(target_id);

                    Ok(json!({ "status": "Blocked" }))
                }
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
        }
    }
}
