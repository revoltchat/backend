use super::Response;
use crate::database::{self, get_relationship, get_relationship_internal, Relationship};
use crate::guards::auth::UserRef;
use crate::routes::channel;

use bson::doc;
use mongodb::options::FindOptions;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// retrieve your user information
#[get("/@me")]
pub fn me(user: UserRef) -> Response {
    if let Some(info) = user.fetch_data(doc! { "email": 1 }) {
        Response::Success(json!({
            "id": user.id,
            "username": user.username,
            "email": info.get_str("email").unwrap(),
            "verified": user.email_verified,
        }))
    } else {
        Response::InternalServerError(
            json!({ "error": "Failed to fetch information from database." }),
        )
    }
}

/// retrieve another user's information
#[get("/<target>")]
pub fn user(user: UserRef, target: UserRef) -> Response {
    Response::Success(json!({
        "id": target.id,
        "username": target.username,
        "relationship": get_relationship(&user, &target) as u8
    }))
}

#[derive(Serialize, Deserialize)]
pub struct Query {
    username: String,
}

/// lookup a user on Revolt
/// currently only supports exact username searches
#[post("/lookup", data = "<query>")]
pub fn lookup(user: UserRef, query: Json<Query>) -> Response {
    let relationships = user.fetch_relationships();
    let col = database::get_collection("users");

    let users = col
        .find(
            doc! { "username": query.username.clone() },
            FindOptions::builder()
                .projection(doc! { "_id": 1, "username": 1 })
                .limit(10)
                .build(),
        )
        .expect("Failed user lookup");

    let mut results = Vec::new();
    for item in users {
        if let Ok(doc) = item {
            let id = doc.get_str("id").unwrap();
            results.push(json!({
                "id": id,
                "username": doc.get_str("username").unwrap(),
                "relationship": get_relationship_internal(&user.id, &id, &relationships) as u8
            }));
        }
    }

    Response::Success(json!(results))
}

/// retrieve all of your DMs
#[get("/@me/dms")]
pub fn dms(user: UserRef) -> Response {
    let col = database::get_collection("channels");

    let results = col
        .find(
            doc! {
                "$or": [
                    {
                        "type": channel::ChannelType::DM as i32
                    },
                    {
                        "type": channel::ChannelType::GROUPDM as i32
                    }
                ],
                "recipients": user.id
            },
            FindOptions::builder().projection(doc! {}).build(),
        )
        .expect("Failed channel lookup");

    let mut channels = Vec::new();
    for item in results {
        if let Ok(doc) = item {
            let id = doc.get_str("_id").unwrap();
            let recipients = doc.get_array("recipients").unwrap();

            match doc.get_i32("type").unwrap() {
                0 => {
                    channels.push(json!({
                        "id": id,
                        "type": 0,
                        "recipients": recipients,
                    }));
                }
                1 => {
                    channels.push(json!({
                        "id": id,
                        "type": 1,
                        "recipients": recipients,
                        "name": doc.get_str("name").unwrap(),
                        "owner": doc.get_str("owner").unwrap(),
                        "description": doc.get_str("description").unwrap_or(""),
                    }));
                }
                _ => unreachable!(),
            }
        }
    }

    Response::Success(json!(channels))
}

/// open a DM with a user
#[get("/<target>/dm")]
pub fn dm(user: UserRef, target: UserRef) -> Response {
    let col = database::get_collection("channels");

    match col.find_one(
		doc! { "type": channel::ChannelType::DM as i32, "recipients": { "$all": [ user.id.clone(), target.id.clone() ] } },
		None
	).expect("Failed channel lookup") {
        Some(channel) =>
            Response::Success( json!({ "id": channel.get_str("_id").unwrap() })),
		None => {
			let id = Ulid::new();

			col.insert_one(
				doc! {
					"_id": id.to_string(),
					"type": channel::ChannelType::DM as i32,
					"recipients": [ user.id, target.id ],
					"active": false
				},
				None
			).expect("Failed insert query.");

            Response::Success(json!({ "id": id.to_string() }))
		}
	}
}

/// retrieve all of your friends
#[get("/@me/friend")]
pub fn get_friends(user: UserRef) -> Response {
    let relationships = user.fetch_relationships();

    let mut results = Vec::new();
    if let Some(arr) = relationships {
        for item in arr {
            results.push(json!({
                "id": item.id,
                "status": item.status
            }))
        }
    }

    Response::Success(json!(results))
}

/// retrieve friend status with user
#[get("/<target>/friend")]
pub fn get_friend(user: UserRef, target: UserRef) -> Response {
    let relationship = get_relationship(&user, &target);

    Response::Success(json!({ "status": relationship as u8 }))
}

/// create or accept a friend request
#[put("/<target>/friend")]
pub fn add_friend(user: UserRef, target: UserRef) -> Response {
    let relationship = get_relationship(&user, &target);
    let col = database::get_collection("users");

    match relationship {
        Relationship::Friend => Response::BadRequest(json!({ "error": "Already friends." })),
        Relationship::Outgoing => {
            Response::BadRequest(json!({ "error": "Already sent a friend request." }))
        }
        Relationship::Incoming => {
            if col
                .update_one(
                    doc! {
                        "_id": user.id.clone(),
                        "relations.id": target.id.clone()
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": Relationship::Friend as i32
                        }
                    },
                    None,
                )
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id,
                            "relations.id": user.id
                        },
                        doc! {
                            "$set": {
                                "relations.$.status": Relationship::Friend as i32
                            }
                        },
                        None,
                    )
                    .is_ok()
                {
                    Response::Success(json!({ "status": Relationship::Friend as u8 }))
                } else {
                    Response::InternalServerError(
                        json!({ "error": "Failed to commit! Try re-adding them as a friend." }),
                    )
                }
            } else {
                Response::InternalServerError(
                    json!({ "error": "Failed to commit to database, try again." }),
                )
            }
        }
        Relationship::Blocked => {
            Response::BadRequest(json!({ "error": "You have blocked this person." }))
        }
        Relationship::BlockedOther => {
            Response::Conflict(json!({ "error": "You have been blocked by this person." }))
        }
        Relationship::NONE => {
            if col
                .update_one(
                    doc! {
                        "_id": user.id.clone()
                    },
                    doc! {
                        "$push": {
                            "relations": {
                                "id": target.id.clone(),
                                "status": Relationship::Outgoing as i32
                            }
                        }
                    },
                    None,
                )
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id
                        },
                        doc! {
                            "$push": {
                                "relations": {
                                    "id": user.id,
                                    "status": Relationship::Incoming as i32
                                }
                            }
                        },
                        None,
                    )
                    .is_ok()
                {
                    Response::Success(json!({ "status": Relationship::Outgoing as u8 }))
                } else {
                    Response::InternalServerError(
                        json!({ "error": "Failed to commit! Try re-adding them as a friend." }),
                    )
                }
            } else {
                Response::InternalServerError(
                    json!({ "error": "Failed to commit to database, try again." }),
                )
            }
        }
        Relationship::SELF => {
            Response::BadRequest(json!({ "error": "You're already friends with yourself, no? c:" }))
        }
    }
}

/// remove a friend or deny a request
#[delete("/<target>/friend")]
pub fn remove_friend(user: UserRef, target: UserRef) -> Response {
    let relationship = get_relationship(&user, &target);
    let col = database::get_collection("users");

    match relationship {
        Relationship::Friend | Relationship::Outgoing | Relationship::Incoming => {
            if col
                .update_one(
                    doc! {
                        "_id": user.id.clone()
                    },
                    doc! {
                        "$pull": {
                            "relations": {
                                "id": target.id.clone()
                            }
                        }
                    },
                    None,
                )
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id
                        },
                        doc! {
                            "$pull": {
                                "relations": {
                                    "id": user.id
                                }
                            }
                        },
                        None,
                    )
                    .is_ok()
                {
                    Response::Result(super::Status::Ok)
                } else {
                    Response::InternalServerError(
                        json!({ "error": "Failed to commit! Target remains in same state." }),
                    )
                }
            } else {
                Response::InternalServerError(
                    json!({ "error": "Failed to commit to database, try again." }),
                )
            }
        }
        Relationship::Blocked
        | Relationship::BlockedOther
        | Relationship::NONE
        | Relationship::SELF => Response::BadRequest(json!({ "error": "This has no effect." })),
    }
}
