use crate::{database::{entities::{User, RelationshipStatus}, get_collection, guards::reference::Ref, permissions::get_relationship}, util::result::Error};
use rocket_contrib::json::JsonValue;
use crate::util::result::Result;
use mongodb::bson::doc;
use futures::try_join;

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
                Ok(_) => Ok(json!({ "status": "Friend" })),
                Err(_) => Err(Error::DatabaseError { operation: "update_one", with: "user" })
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
                Ok(_) => Ok(json!({ "status": "Outgoing" })),
                Err(_) => Err(Error::DatabaseError { operation: "update_one", with: "user" })
            }
        }
    }
}
