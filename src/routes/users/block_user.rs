use crate::{notifications::{events::ClientboundNotification, hive}, util::result::Result};
use crate::{
    database::entities::RelationshipStatus, database::entities::User, database::get_collection,
    database::guards::reference::Ref, database::permissions::get_relationship, util::result::Error,
};
use futures::try_join;
use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[put("/<target>/block")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let col = get_collection("users");

    match get_relationship(&user, &target) {
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
                user: target.id.clone(),
                status: RelationshipStatus::Blocked
            }.publish(user.id.clone()).await.ok();

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
                    try_join!(
                        ClientboundNotification::UserRelationship {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: RelationshipStatus::Blocked
                        }.publish(user.id.clone()),
                        ClientboundNotification::UserRelationship {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: RelationshipStatus::BlockedOther
                        }.publish(target.id.clone())
                    ).ok();

                    hive::subscribe_if_exists(user.id.clone(), target.id.clone()).ok();
                    hive::subscribe_if_exists(target.id.clone(), user.id.clone()).ok();

                    Ok(json!({ "status": "Blocked" }))
                },
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
                    try_join!(
                        ClientboundNotification::UserRelationship {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: RelationshipStatus::Blocked
                        }.publish(user.id.clone()),
                        ClientboundNotification::UserRelationship {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: RelationshipStatus::BlockedOther
                        }.publish(target.id.clone())
                    ).ok();
                    
                    Ok(json!({ "status": "Blocked" }))
                },
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
        }
    }
}
