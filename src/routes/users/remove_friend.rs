use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use futures::try_join;
use mongodb::bson::doc;
use rocket::serde::json::Value;

#[delete("/<target>/friend")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }

    let col = get_collection("users");
    let target = target.fetch_user().await?;

    match get_relationship(&user, &target.id) {
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
                    let target = target
                        .from_override(&user, RelationshipStatus::None)
                        .await?;
                    let user = user
                        .from_override(&target, RelationshipStatus::None)
                        .await?;
                    let target_id = target.id.clone();

                    ClientboundNotification::UserRelationship {
                        id: user.id.clone(),
                        user: target,
                        status: RelationshipStatus::None,
                    }
                    .publish(user.id.clone());

                    ClientboundNotification::UserRelationship {
                        id: target_id.clone(),
                        user,
                        status: RelationshipStatus::None,
                    }
                    .publish(target_id);

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
