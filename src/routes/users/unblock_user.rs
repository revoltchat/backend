use crate::{database::entities::RelationshipStatus, database::guards::reference::Ref, database::entities::User, database::permissions::get_relationship, util::result::Error, database::get_collection};
use rocket_contrib::json::JsonValue;
use crate::util::result::Result;
use mongodb::bson::doc;
use futures::try_join;

#[delete("/<target>/block")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let col = get_collection("users");

    match get_relationship(&user, &target) {
        RelationshipStatus::None |
        RelationshipStatus::User |
        RelationshipStatus::BlockedOther |
        RelationshipStatus::Incoming |
        RelationshipStatus::Outgoing |
        RelationshipStatus::Friend => Err(Error::NoEffect),
        RelationshipStatus::Blocked => {
            match get_relationship(&target.fetch_user().await?, &user.as_ref()) {
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
                        None
                    )
                    .await
                    .map_err(|_| Error::DatabaseError { operation: "update_one", with: "user" })?;
        
                    Ok(json!({ "status": "BlockedOther" }))
                },
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
                        Ok(_) => Ok(json!({ "status": "None" })),
                        Err(_) => Err(Error::DatabaseError { operation: "update_one", with: "user" })
                    }
                },
                _ => Err(Error::InternalError)
            }
        }
    }
}
