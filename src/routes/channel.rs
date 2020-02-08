use crate::database::{ self, user::User, channel::Channel, message::Message };

use bson::{ bson, doc, from_bson, Bson::UtcDatetime };
use rocket_contrib::json::{ JsonValue, Json };
use serde::{ Serialize, Deserialize };
use num_enum::TryFromPrimitive;
use chrono::prelude::*;
use ulid::Ulid;

#[derive(Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum ChannelType {
	DM = 0,
	GROUPDM = 1,
	GUILDCHANNEL = 2,
}

fn has_permission(user: &User, target: &Channel) -> bool {
	match target.channel_type {
		0..=1 => {
			if let Some(arr) = &target.recipients {
				for item in arr {
					if item == &user.id {
						return true;
					}
				}
			}

			false
		},
		2 =>
			false,
		_ =>
			false
	}
}

/// fetch channel information
#[get("/<target>")]
pub fn channel(user: User, target: Channel) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	Some(
		json!({
			"id": target.id,
			"type": target.channel_type
		}
	))
}

/// delete channel
/// or leave group DM
/// or close DM conversation
#[delete("/<target>")]
pub fn delete(user: User, target: Channel) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	let col = database::get_collection("channels");
	Some(match target.channel_type {
		0 => {
			col.update_one(
				doc! { "_id": target.id },
				doc! { "$set": { "active": false } },
				None
			).expect("Failed to update channel.");

			json!({
				"success": true
			})
		},
		1 => {
			// ? TODO: group dm

			json!({
				"success": true
			})
		},
		2 => {
			// ? TODO: guild

			json!({
				"success": true
			})
		},
		_ => 
			json!({
				"success": false
			})
	})
}

/// fetch channel messages
#[get("/<target>/messages")]
pub fn messages(user: User, target: Channel) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	let col = database::get_collection("messages");
	let result = col.find(
		doc! { "channel": target.id },
		None
	).unwrap();

	let mut messages = Vec::new();
	for item in result {
		let message: Message = from_bson(bson::Bson::Document(item.unwrap())).expect("Failed to unwrap message.");
		messages.push(
			json!({
				"id": message.id,
				"author": message.author,
				"content": message.content,
				"edited": if let Some(t) = message.edited { Some(t.timestamp()) } else { None }
			})
		);
	}

	Some(json!(messages))
}

#[derive(Serialize, Deserialize)]
pub struct SendMessage {
	content: String,
}

/// send a message to a channel
#[post("/<target>/messages", data = "<message>")]
pub fn send_message(user: User, target: Channel, message: Json<SendMessage>) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	let col = database::get_collection("messages");
	let id = Ulid::new().to_string();
	Some(match col.insert_one(
		doc! {
			"_id": id.clone(),
			"channel": target.id,
			"author": user.id,
			"content": message.content.clone(),
		},
		None
	) {
		Ok(_) =>
			json!({
				"success": true,
				"id": id
			}),
		Err(_) =>
			json!({
				"success": false,
				"error": "Failed database query."
			})
		})
}

#[derive(Serialize, Deserialize)]
pub struct EditMessage {
	content: String,
}

/// edit a message
#[patch("/<target>/messages/<message>", data = "<edit>")]
pub fn edit_message(user: User, target: Channel, message: Message, edit: Json<SendMessage>) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	Some(
		if message.author != user.id {
			json!({
				"success": false,
				"error": "You did not send this message."
			})
		} else {
			let col = database::get_collection("messages");

			match col.update_one(
				doc! { "_id": message.id },
				doc! {
					"$set": {
						"content": edit.content.clone(),
						"edited": UtcDatetime(Utc::now())
					}
				},
				None
			) {
				Ok(_) =>
					json!({
						"success": true
					}),
				Err(_) =>
					json!({
						"success": false,
						"error": "Failed to update message."
					})
				}
		}
	)
}

/// delete a message
#[delete("/<target>/messages/<message>")]
pub fn delete_message(user: User, target: Channel, message: Message) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	Some(
		if message.author != user.id {
			json!({
				"success": false,
				"error": "You did not send this message."
			})
		} else {
			let col = database::get_collection("messages");

			match col.delete_one(
				doc! { "_id": message.id },
				None
			) {
				Ok(_) =>
					json!({
						"success": true
					}),
				Err(_) =>
					json!({
						"success": false,
						"error": "Failed to delete message."
					})
				}
		}
	)
}
