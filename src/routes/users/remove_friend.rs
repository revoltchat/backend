use crate::util::result::Result;
use crate::{
    database::entities::RelationshipStatus, database::entities::User, database::get_collection,
    database::guards::reference::Ref, database::permissions::get_relationship, util::result::Error,
};
use futures::try_join;
use mongodb::bson::doc;
use rocket_contrib::json::JsonValue;

#[delete("/<target>/friend")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let col = get_collection("users");

    match get_relationship(&user, &target) {
        RelationshipStatus::Blocked
        | RelationshipStatus::BlockedOther
        | RelationshipStatus::User
        | RelationshipStatus::None => Err(Error::NoEffect),
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
                Ok(_) => Ok(json!({ "status": "None" })),
                Err(_) => Err(Error::DatabaseError {
                    operation: "update_one",
                    with: "user",
                }),
            }
        }
    }
}
