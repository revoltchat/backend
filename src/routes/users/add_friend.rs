use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use futures::try_join;
use mongodb::bson::doc;
use mongodb::options::{Collation, FindOneOptions};
use rocket::serde::json::Value;

#[put("/<username>/friend")]
pub async fn req(user: User, username: String) -> Result<Value> {
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

    let target_user = Ref::from(target_id.to_string())?.fetch_user().await?;

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
                    let target_user = target_user
                        .from_override(&user, RelationshipStatus::Friend)
                        .await?;
                    let user = user
                        .from_override(&target_user, RelationshipStatus::Friend)
                        .await?;

                    ClientboundNotification::UserRelationship {
                        id: user.id.clone(),
                        user: target_user,
                        status: RelationshipStatus::Friend,
                    }
                    .publish(user.id.clone());

                    ClientboundNotification::UserRelationship {
                        id: target_id.to_string(),
                        user,
                        status: RelationshipStatus::Friend,
                    }
                    .publish(target_id.to_string());

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
                    let target_user = target_user
                        .from_override(&user, RelationshipStatus::Outgoing)
                        .await?;
                    let user = user
                        .from_override(&target_user, RelationshipStatus::Incoming)
                        .await?;

                    ClientboundNotification::UserRelationship {
                        id: user.id.clone(),
                        user: target_user,
                        status: RelationshipStatus::Outgoing,
                    }
                    .publish(user.id.clone());

                    ClientboundNotification::UserRelationship {
                        id: target_id.to_string(),
                        user,
                        status: RelationshipStatus::Incoming,
                    }
                    .publish(target_id.to_string());

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
