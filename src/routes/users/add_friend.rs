use crate::database::*;
use crate::notifications::{events::ClientboundNotification, hive};
use crate::util::result::{Error, Result};

use futures::try_join;
use mongodb::bson::doc;
use mongodb::options::{Collation, FindOneOptions};
use rocket_contrib::json::JsonValue;

#[put("/<username>/friend")]
pub async fn req(user: User, username: String) -> Result<JsonValue> {
    let col = get_collection("users");
    let doc = col
        .find_one(
            doc! {
                "username": username
            },
            FindOneOptions::builder()
                .collation(Collation::builder().locale("en").strength(2).build())
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "user",
        })?
        .ok_or_else(|| Error::UnknownUser)?;

    let target_id = doc.get_str("_id").map_err(|_| Error::DatabaseError {
        operation: "get_str(_id)",
        with: "user",
    })?;

    match get_relationship(&user, &target_id) {
        RelationshipStatus::User => return Err(Error::NoEffect),
        RelationshipStatus::Friend => return Err(Error::AlreadyFriends),
        RelationshipStatus::Outgoing => return Err(Error::AlreadySentRequest),
        RelationshipStatus::Blocked => return Err(Error::Blocked),
        RelationshipStatus::BlockedOther => return Err(Error::BlockedByOther),
        RelationshipStatus::Incoming => {
            match try_join!(
                col.update_one(
                    doc! {
                        "_id": &user.id,
                        "relations._id": target_id
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": "Friend"
                        }
                    },
                    None
                ),
                col.update_one(
                    doc! {
                        "_id": target_id,
                        "relations._id": &user.id
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": "Friend"
                        }
                    },
                    None
                )
            ) {
                Ok(_) => {
                    try_join!(
                        ClientboundNotification::UserRelationship {
                            id: user.id.clone(),
                            user: target_id.to_string(),
                            status: RelationshipStatus::Friend
                        }
                        .publish(user.id.clone()),
                        ClientboundNotification::UserRelationship {
                            id: target_id.to_string(),
                            user: user.id.clone(),
                            status: RelationshipStatus::Friend
                        }
                        .publish(target_id.to_string())
                    )
                    .ok();

                    Ok(json!({ "status": "Friend" }))
                }
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
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
                                "_id": target_id,
                                "status": "Outgoing"
                            }
                        }
                    },
                    None
                ),
                col.update_one(
                    doc! {
                        "_id": target_id
                    },
                    doc! {
                        "$push": {
                            "relations": {
                                "_id": &user.id,
                                "status": "Incoming"
                            }
                        }
                    },
                    None
                )
            ) {
                Ok(_) => {
                    try_join!(
                        ClientboundNotification::UserRelationship {
                            id: user.id.clone(),
                            user: target_id.to_string(),
                            status: RelationshipStatus::Outgoing
                        }
                        .publish(user.id.clone()),
                        ClientboundNotification::UserRelationship {
                            id: target_id.to_string(),
                            user: user.id.clone(),
                            status: RelationshipStatus::Incoming
                        }
                        .publish(target_id.to_string())
                    )
                    .ok();

                    Ok(json!({ "status": "Outgoing" }))
                }
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
        }
    }
}
