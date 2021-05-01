use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use futures::try_join;
use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[delete("/<target>/block")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let col = get_collection("users");
    let target = target.fetch_user().await?;

    match get_relationship(&user, &target.id) {
        RelationshipStatus::Blocked => match get_relationship(&target, &user.id) {
            RelationshipStatus::Blocked => {
                col.update_one(
                    doc! {
                        "_id": &user.id,
                        "relations._id": &target.id
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": "BlockedOther"
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                })?;

                let target = target
                    .from_override(&user, RelationshipStatus::BlockedOther)
                    .await?;
                ClientboundNotification::UserRelationship {
                    id: user.id.clone(),
                    user: target,
                    status: RelationshipStatus::BlockedOther,
                }
                .publish(user.id.clone())
                .await
                .ok();

                Ok(json!({ "status": "BlockedOther" }))
            }
            RelationshipStatus::BlockedOther => {
                match try_join!(
                    col.update_one(
                        doc! {
                            "_id": &user.id
                        },
                        doc! {
                            "$pull": {
                                "relations": {
                                    "_id": &target.id
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
                            "$pull": {
                                "relations": {
                                    "_id": &user.id
                                }
                            }
                        },
                        None
                    )
                ) {
                    Ok(_) => {
                        let target = target
                            .from_override(&user, RelationshipStatus::None)
                            .await?;
                        let user = user
                            .from_override(&target, RelationshipStatus::None)
                            .await?;
                        let target_id = target.id.clone();

                        try_join!(
                            ClientboundNotification::UserRelationship {
                                id: user.id.clone(),
                                user: target,
                                status: RelationshipStatus::None
                            }
                            .publish(user.id.clone()),
                            ClientboundNotification::UserRelationship {
                                id: target_id.clone(),
                                user: user,
                                status: RelationshipStatus::None
                            }
                            .publish(target_id)
                        )
                        .ok();

                        Ok(json!({ "status": "None" }))
                    }
                    Err(_) => Err(Error::DatabaseError {
                        operation: "update_one",
                        with: "user",
                    }),
                }
            }
            _ => Err(Error::InternalError),
        },
        _ => Err(Error::NoEffect),
    }
}
