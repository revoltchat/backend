use crate::database::{ self, user::User, channel::Channel };
use crate::routes::channel;

use rocket_contrib::json::{ Json, JsonValue };
use serde::{ Serialize, Deserialize };
use bson::{ bson, doc, from_bson };
use mongodb::options::FindOptions;
use ulid::Ulid;

/// retrieve your user information
#[get("/@me")]
pub fn me(user: User) -> JsonValue {
	json!({
		"id": user.id,
		"username": user.username,
		"email": user.email,
		"verified": user.email_verification.verified,
	})
}

/// retrieve another user's information
#[get("/<target>")]
pub fn user(user: User, target: User) -> JsonValue {
	json!({
		"id": target.id,
		"username": target.username,
		"relationship": get_relationship(&user, &target) as u8
	})
}

#[derive(Serialize, Deserialize)]
pub struct Query {
	username: String,
}

/// lookup a user on Revolt
/// currently only supports exact username searches
#[post("/lookup", data = "<query>")]
pub fn lookup(user: User, query: Json<Query>) -> JsonValue {
	let col = database::get_collection("users");

	let users = col.find(
		doc! { "username": query.username.clone() },
		FindOptions::builder().limit(10).build()
	).expect("Failed user lookup");

	let mut results = Vec::new();
	for item in users {
		let u: User = from_bson(bson::Bson::Document(item.unwrap())).expect("Failed to unwrap user.");
		results.push(
			json!({
				"id": u.id,
				"username": u.username,
				"relationship": get_relationship(&user, &u) as u8
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
					"type": channel::ChannelType::GROUPDM as i32
				}
			],
			"recipients": user.id
		},
		None
	).expect("Failed channel lookup");

	let mut channels = Vec::new();
	for item in results {
		let channel: Channel = from_bson(bson::Bson::Document(item.unwrap())).expect("Failed to unwrap channel.");
		
		channels.push(
			json!({
				"id": channel.id,
				"type": channel.channel_type,
				"recipients": channel.recipients,
				"active": channel.active.unwrap()
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
		doc! { "type": channel::ChannelType::DM as i32, "recipients": { "$all": [ user.id.clone(), target.id.clone() ] } },
		None
	).expect("Failed channel lookup") {
		Some(channel) =>
			json!({
				"success": true,
				"id": channel.get_str("_id").unwrap()
			}),
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
	BLOCKEDOTHER = 4,
	NONE = 5,
	SELF = 6,
}

fn get_relationship(a: &User, b: &User) -> Relationship {
	if a.id == b.id {
		return Relationship::SELF
	}

	if let Some(arr) = &b.relations {
		for entry in arr {
			if entry.id == a.id {
				match entry.status {
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
						return Relationship::BLOCKEDOTHER
					},
					4 => {
						return Relationship::BLOCKED
					},
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
	if let Some(arr) = user.relations {
		for item in arr {
			results.push(
				json!({
					"id": item.id,
					"status": item.status
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
		"id": target.id,
		"status": relationship as u8
	})
}

/// create or accept a friend request
#[put("/<target>/friend")]
pub fn add_friend(user: User, target: User) -> JsonValue {
	let col = database::get_collection("users");
	let relationship = get_relationship(&user, &target);

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
					"_id": user.id.clone(),
					"relations.id": target.id.clone()
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
					"_id": target.id,
					"relations.id": user.id
				},
				doc! {
					"$set": {
						"relations.$.status": Relationship::FRIEND as i32
					}
				},
				None
			).expect("Failed update query.");

			json!({
				"success": true,
				"status": Relationship::FRIEND as u8,
			})
		},
		Relationship::BLOCKED =>
			json!({
				"success": false,
				"error": "You have blocked this person."
			}),
		Relationship::BLOCKEDOTHER =>
			json!({
				"success": false,
				"error": "You have been blocked by this person."
			}),
		Relationship::NONE => {
			col.update_one(
				doc! {
					"_id": user.id.clone()
				},
				doc! {
					"$push": {
						"relations": {
							"id": target.id.clone(),
							"status": Relationship::OUTGOING as i32
						}
					}
				},
				None
			).expect("Failed update query.");
			
			col.update_one(
				doc! {
					"_id": target.id
				},
				doc! {
					"$push": {
						"relations": {
							"id": user.id,
							"status": Relationship::INCOMING as i32
						}
					}
				},
				None
			).expect("Failed update query.");

			json!({
				"success": true,
				"status": Relationship::OUTGOING as u8,
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

	match relationship {
		Relationship::FRIEND |
		Relationship::OUTGOING |
		Relationship::INCOMING => {
			col.update_one(
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
				None
			).expect("Failed update query.");

			col.update_one(
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
				None
			).expect("Failed update query.");

			json!({
				"success": true
			})
		},
		Relationship::BLOCKED |
		Relationship::BLOCKEDOTHER |
		Relationship::NONE |
		Relationship::SELF =>
			json!({
				"success": false,
				"error": "This has no effect."
			})
	}
}
