use crate::database::*;
use crate::notifications::{events::ClientboundNotification, hive};
use crate::util::result::{Error, Result};

use futures::try_join;
use hive_pubsub::PubSub;
use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[delete("/<target>/friend")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let col = get_collection("users");

    match get_relationship(&user, &target) {
        RelationshipStatus::Friend
        | RelationshipStatus::Outgoing
        | RelationshipStatus::Incoming => {
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
                    try_join!(
                        ClientboundNotification::UserRelationship {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: RelationshipStatus::None
                        }
                        .publish(user.id.clone()),
                        ClientboundNotification::UserRelationship {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: RelationshipStatus::None
                        }
                        .publish(target.id.clone())
                    )
                    .ok();

                    let hive = hive::get_hive();
                    hive.unsubscribe(&user.id, &target.id).ok();
                    hive.unsubscribe(&target.id, &user.id).ok();

                    Ok(json!({ "status": "None" }))
                }
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
        }
        _ => Err(Error::NoEffect),
    }
}
