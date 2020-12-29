use crate::{database::entities::RelationshipStatus, database::guards::reference::Ref, database::entities::User, database::permissions::get_relationship, util::result::Error, database::get_collection};
use rocket_contrib::json::JsonValue;
use crate::util::result::Result;
use mongodb::bson::doc;
use futures::try_join;

#[put("/<target>/block")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let col = get_collection("users");

    match get_relationship(&user, &target) {
        RelationshipStatus::User |
        RelationshipStatus::Blocked => Err(Error::NoEffect),
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
                None
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "user" })?;

            Ok(json!({ "status": "Blocked" }))
        },
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
                Ok(_) => Ok(json!({ "status": "Blocked" })),
                Err(_) => Err(Error::DatabaseError { operation: "update_one", with: "user" })
            }
        },
        RelationshipStatus::Friend |
        RelationshipStatus::Incoming |
        RelationshipStatus::Outgoing => {
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
                Ok(_) => Ok(json!({ "status": "Blocked" })),
                Err(_) => Err(Error::DatabaseError { operation: "update_one", with: "user" })
            }
        }
    }
}
