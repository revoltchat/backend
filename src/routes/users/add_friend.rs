use crate::{notifications::{events::ClientboundNotification, hive}, util::result::Result};
use crate::{
    database::{
        entities::{RelationshipStatus, User},
        get_collection,
        guards::reference::Ref,
        permissions::get_relationship,
    },
    util::result::Error,
};
use futures::try_join;
use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[put("/<target>/friend")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let col = get_collection("users");

    match get_relationship(&user, &target) {
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
                        "relations._id": &target.id
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
                        "_id": &target.id,
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
                            user: target.id.clone(),
                            status: RelationshipStatus::Friend
                        }.publish(user.id.clone()),
                        ClientboundNotification::UserRelationship {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: RelationshipStatus::Friend
                        }.publish(target.id.clone())
                    ).ok();

                    Ok(json!({ "status": "Friend" }))
                },
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
                                "_id": &target.id,
                                "status": "Outgoing"
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
                            user: target.id.clone(),
                            status: RelationshipStatus::Outgoing
                        }.publish(user.id.clone()),
                        ClientboundNotification::UserRelationship {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: RelationshipStatus::Incoming
                        }.publish(target.id.clone())
                    ).ok();

                    hive::subscribe_if_exists(user.id.clone(), target.id.clone()).ok();
                    hive::subscribe_if_exists(target.id.clone(), user.id.clone()).ok();

                    Ok(json!({ "status": "Outgoing" }))
                },
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
        }
    }
}
