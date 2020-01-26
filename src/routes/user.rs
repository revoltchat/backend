use crate::auth::User;
use crate::database;
use crate::routes::channel;

use rocket_contrib::json::{ Json, JsonValue };
use serde::{ Serialize, Deserialize };
use mongodb::options::FindOptions;
use bson::{ bson, doc };
use ulid::Ulid;

/// retrieve your user information
#[get("/@me")]
pub fn me(user: User) -> JsonValue {
	let User ( id, username, doc ) = user;

	json!({
		"id": id.to_string(),
		"username": username,
		"email": doc.get_str("email").expect("Missing email in user object!"),
		"verified": doc.get_document("email_verification").expect("DOC[email_verification]")
						.get_bool("verified").expect("DOC[verified]"),
		"created_timestamp": id.datetime().timestamp(),
	})
}

/// retrieve another user's information
#[get("/<target>")]
pub fn user(user: User, target: User) -> JsonValue {
	json!([])
}

#[derive(Serialize, Deserialize)]
pub struct Query {
	username: String,
}

/// lookup a user on Revolt
/// currently only supports exact username searches
#[post("/lookup", data = "<query>")]
pub fn lookup(_user: User, query: Json<Query>) -> JsonValue {
	let col = database::get_collection("users");

	let users = col.find(
		doc! { "username": query.username.clone() },
		FindOptions::builder().limit(10).build()
	).expect("Failed user lookup");

	let mut results = Vec::new();
	for user in users {
		let u = user.expect("Failed to unwrap user.");
		results.push(
			json!({
				"id": u.get_str("_id").expect("DB[id]"),
				"username": u.get_str("username").expect("DB[username]")
			})
		);
	}

	json!(results)
}

/// retrieve all of your DMs
#[get("/@me/dms")]
pub fn dms(user: User) -> JsonValue {
	let col = database::get_collection("channels");

	let results = col.find(
		doc! {
			"$or": [
				{
					"type": channel::ChannelType::DM as i32
				},
				{
					"type": channel::ChannelType::GROUP_DM as i32
				}
			],
			"recipients": user.0.to_string()
		},
		None
	).expect("Failed channel lookup");

	let mut channels = Vec::new();
	for res in results {
		let doc = res.expect("Failed to unwrap document");

		let mut recipients = Vec::new();
		for user in doc.get_array("recipients").expect("DB[recipients]") {
			recipients.push(
				user.as_str()
					.expect("Should be a string.")
			);
		}

		let active =
			match doc.get_bool("active") {
				Ok(x) => x,
				Err(_) => true
			};
		
		channels.push(
			json!({
				"id": doc.get_str("_id").expect("DB[id]"),
				"type": doc.get_i32("type").expect("DB[type]"),
				"recipients": recipients,
				"active": active
			})
		);
	}

	json!(channels)
}

/// open a DM with a user
#[get("/<target>/dm")]
pub fn dm(user: User, target: User) -> JsonValue {
	let col = database::get_collection("channels");

	match col.find_one(
		doc! { "type": channel::ChannelType::DM as i32, "recipients": [ user.0.to_string(), target.0.to_string() ] },
		None
	).expect("Failed channel lookup") {
		Some(channel) =>
			json!({
				"id": channel.get_str("_id").expect("DB[id]")
			}),
		None => {
			let id = Ulid::new();

			col.insert_one(
				doc! {
					"_id": id.to_string(),
					"type": channel::ChannelType::DM as i32,
					"recipients": [ user.0.to_string(), target.0.to_string() ],
					"active": false
				},
				None
			).expect("Failed insert query.");

			json!({
				"id": id.to_string()
			})
		}
	}
}

enum Relationship {
	FRIEND = 0,
	OUTGOING = 1,
	INCOMING = 2,
	BLOCKED = 3,
	BLOCKED_OTHER = 4,
	NONE = 5,
	SELF = 6,
}

fn get_relationship(a: &User, b: &User) -> Relationship {
	if a.0.to_string() == b.0.to_string() {
		return Relationship::SELF
	}

	if let Ok(arr) = b.2.get_array("relations") {
		let id = a.0.to_string();
		
		for entry in arr {
			let relation = entry.as_document().expect("Expected document in relations array.");

			if relation.get_str("id").expect("DB[id]") == id {

				match relation.get_i32("status").expect("DB[status]") {
					0 => {
						return Relationship::FRIEND
					},
					1 => {
						return Relationship::INCOMING
					},
					2 => {
						return Relationship::OUTGOING
					},
					3 => {
						return Relationship::BLOCKED_OTHER
					}
					_ => {
						return Relationship::NONE
					}
				}

			}
		}
	}

	Relationship::NONE
}

/// retrieve all of your friends
#[get("/@me/friend")]
pub fn get_friends(user: User) -> JsonValue {
	let mut results = Vec::new();
	if let Ok(arr) = user.2.get_array("relations") {
		for item in arr {
			let doc = item.as_document().expect("Expected document in relations array.");
			results.push(
				json!({
					"id": doc.get_str("id").expect("DB[id]"),
					"status": doc.get_i32("status").expect("DB[status]")
				})
			)
		}
	}
	
	json!(results)
}

/// retrieve friend status with user
#[get("/<target>/friend")]
pub fn get_friend(user: User, target: User) -> JsonValue {
	let relationship = get_relationship(&user, &target);

	json!({
		"id": target.0.to_string(),
		"status": relationship as u8
	})
}

/// create or accept a friend request
#[put("/<target>/friend")]
pub fn add_friend(user: User, target: User) -> JsonValue {
	let col = database::get_collection("users");

	let relationship = get_relationship(&user, &target);
	let User ( id, _, _ ) = user;
	let User ( tid, _, _ ) = target;

	match relationship {
		Relationship::FRIEND =>
			json!({
				"success": false,
				"error": "Already friends."
			}),
		Relationship::OUTGOING =>
			json!({
				"success": false,
				"error": "Already sent a friend request."
			}),
		Relationship::INCOMING => {
			col.update_one(
				doc! {
					"_id": id.to_string(),
					"relations.id": tid.to_string()
				},
				doc! {
					"$set": {
						"relations.$.status": Relationship::FRIEND as i32
					}
				},
				None
			).expect("Failed update query.");
			
			col.update_one(
				doc! {
					"_id": tid.to_string(),
					"relations.id": id.to_string()
				},
				doc! {
					"$set": {
						"relations.$.status": Relationship::FRIEND as i32
					}
				},
				None
			).expect("Failed update query.");

			json!({
				"success": true
			})
		},
		Relationship::BLOCKED =>
			json!({
				"success": false,
				"error": "You have blocked this person."
			}),
		Relationship::BLOCKED_OTHER =>
			json!({
				"success": false,
				"error": "You have been blocked by this person."
			}),
		Relationship::NONE => {
			col.update_one(
				doc! {
					"_id": id.to_string()
				},
				doc! {
					"$push": {
						"relations": {
							"id": tid.to_string(),
							"status": Relationship::OUTGOING as i32
						}
					}
				},
				None
			).expect("Failed update query.");
			
			col.update_one(
				doc! {
					"_id": tid.to_string()
				},
				doc! {
					"$push": {
						"relations": {
							"id": id.to_string(),
							"status": Relationship::INCOMING as i32
						}
					}
				},
				None
			).expect("Failed update query.");

			json!({
				"success": true
			})
		},
		Relationship::SELF =>
			json!({
				"success": false,
				"error": "Cannot add yourself as a friend."
			})
	}
}

/// remove a friend or deny a request
#[delete("/<target>/friend")]
pub fn remove_friend(user: User, target: User) -> JsonValue {
	let col = database::get_collection("users");

	let relationship = get_relationship(&user, &target);
	let User ( id, _, _ ) = user;
	let User ( tid, _, _ ) = target;

	match relationship {
		Relationship::FRIEND |
		Relationship::OUTGOING |
		Relationship::INCOMING => {
			col.update_one(
				doc! {
					"_id": id.to_string()
				},
				doc! {
					"$pull": {
						"relations": {
							"id": tid.to_string()
						}
					}
				},
				None
			).expect("Failed update query.");

			col.update_one(
				doc! {
					"_id": tid.to_string()
				},
				doc! {
					"$pull": {
						"relations": {
							"id": id.to_string()
						}
					}
				},
				None
			).expect("Failed update query.");

			json!({
				"success": true
			})
		},
		Relationship::BLOCKED |
		Relationship::BLOCKED_OTHER |
		Relationship::NONE |
		Relationship::SELF =>
			json!({
				"success": false,
				"error": "This has no effect."
			})
	}
}
