use super::Response;
use crate::database::{
    self, get_relationship, get_relationship_internal, user::User, Relationship,
};
use crate::routes::channel;

use mongodb::bson::doc;
use mongodb::options::{Collation, FindOneOptions, FindOptions};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use rocket::futures::StreamExt;
use ulid::Ulid;

/// retrieve your user information
#[get("/@me")]
pub async fn me(user: User) -> Response {
    Response::Success(user.serialise(Relationship::SELF as i32))
}

/// retrieve another user's information
#[get("/<target>")]
pub async fn user(user: User, target: User) -> Response {
    let relationship = get_relationship(&user, &target) as i32;
    Response::Success(user.serialise(relationship))
}

#[derive(Serialize, Deserialize)]
pub struct UserQuery {
    username: String,
}

/// find a user by their username
#[post("/query", data = "<query>")]
pub async fn query(user: User, query: Json<UserQuery>) -> Response {
    let col = database::get_collection("users");

    if let Ok(result) = col.find_one(
        doc! { "username": query.username.clone() },
        FindOneOptions::builder()
            .collation(Collation::builder().locale("en").strength(2).build())
            .build(),
    )
    .await {
        if let Some(doc) = result {
            let id = doc.get_str("_id").unwrap();
            Response::Success(json!({
                "id": id,
                "username": doc.get_str("username").unwrap(),
                "display_name": doc.get_str("display_name").unwrap(),
                "relationship": get_relationship_internal(&user.id, &id, &user.relations) as i32
            }))
        } else {
            Response::NotFound(json!({
                "error": "User not found!"
            }))
        }
    } else {
        Response::InternalServerError(json!({ "error": "Failed database query." }))
    }
}

/*#[derive(Serialize, Deserialize)]
pub struct LookupQuery {
    username: String,
}

/// lookup a user on Revolt
/// currently only supports exact username searches
#[post("/lookup", data = "<query>")]
pub fn lookup(user: User, query: Json<LookupQuery>) -> Response {
    let relationships = user.fetch_relationships();
    let col = database::get_collection("users");

    if let Ok(users) = col.find(
        doc! { "username": query.username.clone() },
        FindOptions::builder()
            .projection(doc! { "_id": 1, "username": 1 })
            .limit(10)
            .build(),
    ) {
        let mut results = Vec::new();
        for item in users {
            if let Ok(doc) = item {
                let id = doc.get_str("id").unwrap();
                results.push(json!({
                    "id": id,
                    "username": doc.get_str("username").unwrap(),
                    "relationship": get_relationship_internal(&user.id, &id, &relationships) as i32
                }));
            }
        }

        Response::Success(json!(results))
    } else {
        Response::InternalServerError(json!({ "error": "Failed database query." }))
    }
}*/

/// retrieve all of your DMs
#[get("/@me/dms")]
pub async fn dms(user: User) -> Response {
    let col = database::get_collection("channels");

    if let Ok(mut results) = col.find(
        doc! {
            "$or": [
                {
                    "type": channel::ChannelType::DM as i32,
                    "active": true
                },
                {
                    "type": channel::ChannelType::GROUPDM as i32
                }
            ],
            "recipients": user.id
        },
        FindOptions::builder().projection(doc! {}).build(),
    )
    .await {
        let mut channels = Vec::new();
        while let Some(item) = results.next().await {
            if let Ok(doc) = item {
                let id = doc.get_str("_id").unwrap();
                let last_message = doc.get_document("last_message").unwrap();
                let recipients = doc.get_array("recipients").unwrap();

                match doc.get_i32("type").unwrap() {
                    0 => {
                        channels.push(json!({
                            "id": id,
                            "type": 0,
                            "last_message": last_message,
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
    } else {
        Response::InternalServerError(json!({ "error": "Failed database query." }))
    }
}

/// open a DM with a user
#[get("/<target>/dm")]
pub async fn dm(user: User, target: User) -> Response {
    let col = database::get_collection("channels");

    if let Ok(result) = col.find_one(
		doc! { "type": channel::ChannelType::DM as i32, "recipients": { "$all": [ user.id.clone(), target.id.clone() ] } },
		None
	)
    .await {
        if let Some(channel) = result {
            Response::Success( json!({ "id": channel.get_str("_id").unwrap() }))
        } else {
			let id = Ulid::new();

			if col.insert_one(
				doc! {
					"_id": id.to_string(),
					"type": channel::ChannelType::DM as i32,
					"recipients": [ user.id, target.id ],
					"active": false
				},
				None
			)
            .await
            .is_ok() {
                Response::Success(json!({ "id": id.to_string() }))
            } else {
                Response::InternalServerError(json!({ "error": "Failed to create new channel." }))
            }
		}
	} else {
        Response::InternalServerError(json!({ "error": "Failed server query." }))
    }
}

/// retrieve all of your friends
#[get("/@me/friend")]
pub async fn get_friends(user: User) -> Response {
    let mut results = Vec::new();
    if let Some(arr) = user.relations {
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
pub async fn get_friend(user: User, target: User) -> Response {
    Response::Success(json!({ "status": get_relationship(&user, &target) as i32 }))
}

/// create or accept a friend request
#[put("/<target>/friend")]
pub async fn add_friend(user: User, target: User) -> Response {
    let col = database::get_collection("users");

    match get_relationship(&user, &target) {
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
                .await
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id.clone(),
                            "relations.id": user.id.clone()
                        },
                        doc! {
                            "$set": {
                                "relations.$.status": Relationship::Friend as i32
                            }
                        },
                        None,
                    )
                    .await
                    .is_ok()
                {
                    /*notifications::send_message_threaded(
                        vec![target.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: Relationship::Friend as i32,
                        }),
                    );

                    notifications::send_message_threaded(
                        vec![user.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: Relationship::Friend as i32,
                        }), FIXME
                    );*/

                    Response::Success(json!({ "status": Relationship::Friend as i32 }))
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
                .await
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id.clone()
                        },
                        doc! {
                            "$push": {
                                "relations": {
                                    "id": user.id.clone(),
                                    "status": Relationship::Incoming as i32
                                }
                            }
                        },
                        None,
                    )
                    .await
                    .is_ok()
                {
                    /*notifications::send_message_threaded(
                        vec![user.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: Relationship::Outgoing as i32,
                        }),
                    );

                    notifications::send_message_threaded(
                        vec![target.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: Relationship::Incoming as i32,
                        }), FIXME
                    );*/

                    Response::Success(json!({ "status": Relationship::Outgoing as i32 }))
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
pub async fn remove_friend(user: User, target: User) -> Response {
    let col = database::get_collection("users");

    match get_relationship(&user, &target) {
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
                .await
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id.clone()
                        },
                        doc! {
                            "$pull": {
                                "relations": {
                                    "id": user.id.clone()
                                }
                            }
                        },
                        None,
                    )
                    .await
                    .is_ok()
                {
                    /*notifications::send_message_threaded(
                        vec![user.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: Relationship::NONE as i32,
                        }),
                    );

                    notifications::send_message_threaded(
                        vec![target.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: Relationship::NONE as i32,
                        }), FIXME
                    );*/

                    Response::Success(json!({ "status": Relationship::NONE as i32 }))
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

/// block a user
#[put("/<target>/block")]
pub async fn block_user(user: User, target: User) -> Response {
    let col = database::get_collection("users");

    match get_relationship(&user, &target) {
        Relationship::Friend | Relationship::Incoming | Relationship::Outgoing => {
            if col
                .update_one(
                    doc! {
                        "_id": user.id.clone(),
                        "relations.id": target.id.clone()
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": Relationship::Blocked as i32
                        }
                    },
                    None,
                )
                .await
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id.clone(),
                            "relations.id": user.id.clone()
                        },
                        doc! {
                            "$set": {
                                "relations.$.status": Relationship::BlockedOther as i32
                            }
                        },
                        None,
                    )
                    .await
                    .is_ok()
                {
                    /*notifications::send_message_threaded(
                        vec![user.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: Relationship::Blocked as i32,
                        }),
                    );

                    notifications::send_message_threaded(
                        vec![target.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: Relationship::BlockedOther as i32,
                        }), FIXME
                    );*/

                    Response::Success(json!({ "status": Relationship::Blocked as i32 }))
                } else {
                    Response::InternalServerError(
                        json!({ "error": "Failed to commit! Try blocking the user again, remove it first." }),
                    )
                }
            } else {
                Response::InternalServerError(
                    json!({ "error": "Failed to commit to database, try again." }),
                )
            }
        }

        Relationship::NONE => {
            if col
                .update_one(
                    doc! {
                        "_id": user.id.clone(),
                    },
                    doc! {
                        "$push": {
                            "relations": {
                                "id": target.id.clone(),
                                "status": Relationship::Blocked as i32,
                            }
                        }
                    },
                    None,
                )
                .await
                .is_ok()
            {
                if col
                    .update_one(
                        doc! {
                            "_id": target.id.clone(),
                        },
                        doc! {
                            "$push": {
                                "relations": {
                                    "id": user.id.clone(),
                                    "status": Relationship::BlockedOther as i32,
                                }
                            }
                        },
                        None,
                    )
                    .await
                    .is_ok()
                {
                    /*notifications::send_message_threaded(
                        vec![user.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: Relationship::Blocked as i32,
                        }),
                    );

                    notifications::send_message_threaded(
                        vec![target.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: target.id.clone(),
                            user: user.id.clone(),
                            status: Relationship::BlockedOther as i32,
                        }), FIXME
                    );*/

                    Response::Success(json!({ "status": Relationship::Blocked as i32 }))
                } else {
                    Response::InternalServerError(
                        json!({ "error": "Failed to commit! Try blocking the user again, remove it first." }),
                    )
                }
            } else {
                Response::InternalServerError(
                    json!({ "error": "Failed to commit to database, try again." }),
                )
            }
        }
        Relationship::Blocked => {
            Response::BadRequest(json!({ "error": "Already blocked this person." }))
        }
        Relationship::BlockedOther => {
            if col
                .update_one(
                    doc! {
                        "_id": user.id.clone(),
                        "relations.id": target.id.clone()
                    },
                    doc! {
                        "$set": {
                            "relations.$.status": Relationship::Blocked as i32
                        }
                    },
                    None,
                )
                .await
                .is_ok()
            {
                /*notifications::send_message_threaded(
                    vec![user.id.clone()],
                    None,
                    Notification::user_friend_status(FriendStatus {
                        id: user.id.clone(),
                        user: target.id.clone(),
                        status: Relationship::Blocked as i32,
                    }), FIXME
                );*/

                Response::Success(json!({ "status": Relationship::Blocked as i32 }))
            } else {
                Response::InternalServerError(
                    json!({ "error": "Failed to commit to database, try again." }),
                )
            }
        }
        Relationship::SELF => Response::BadRequest(json!({ "error": "This has no effect." })),
    }
}

/// unblock a user
#[delete("/<target>/block")]
pub async fn unblock_user(user: User, target: User) -> Response {
    let col = database::get_collection("users");

    match get_relationship(&user, &target) {
        Relationship::Blocked => match get_relationship(&target, &user) {
            Relationship::Blocked => {
                if col
                    .update_one(
                        doc! {
                            "_id": user.id.clone(),
                            "relations.id": target.id.clone()
                        },
                        doc! {
                            "$set": {
                                "relations.$.status": Relationship::BlockedOther as i32
                            }
                        },
                        None,
                    )
                    .await
                    .is_ok()
                {
                    /*notifications::send_message_threaded(
                        vec![user.id.clone()],
                        None,
                        Notification::user_friend_status(FriendStatus {
                            id: user.id.clone(),
                            user: target.id.clone(),
                            status: Relationship::BlockedOther as i32,
                        }), FIXME
                    );*/

                    Response::Success(json!({ "status": Relationship::BlockedOther as i32 }))
                } else {
                    Response::InternalServerError(
                        json!({ "error": "Failed to commit to database, try again." }),
                    )
                }
            }
            Relationship::BlockedOther => {
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
                    .await
                    .is_ok()
                {
                    if col
                        .update_one(
                            doc! {
                                "_id": target.id.clone()
                            },
                            doc! {
                                "$pull": {
                                    "relations": {
                                        "id": user.id.clone()
                                    }
                                }
                            },
                            None,
                        )
                        .await
                        .is_ok()
                    {
                        /*notifications::send_message_threaded(
                            vec![user.id.clone()],
                            None,
                            Notification::user_friend_status(FriendStatus {
                                id: user.id.clone(),
                                user: target.id.clone(),
                                status: Relationship::NONE as i32,
                            }),
                        );

                        notifications::send_message_threaded(
                            vec![target.id.clone()],
                            None,
                            Notification::user_friend_status(FriendStatus {
                                id: target.id.clone(),
                                user: user.id.clone(),
                                status: Relationship::NONE as i32,
                            }), FIXME
                        );*/

                        Response::Success(json!({ "status": Relationship::NONE as i32 }))
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
            _ => unreachable!(),
        },
        Relationship::BlockedOther => {
            Response::BadRequest(json!({ "error": "Cannot remove block by other user." }))
        }
        Relationship::Friend
        | Relationship::Incoming
        | Relationship::Outgoing
        | Relationship::SELF
        | Relationship::NONE => Response::BadRequest(json!({ "error": "This has no effect." })),
    }
}
