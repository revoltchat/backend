use crate::database::{ self, user::User, channel::Channel, message::Message };
use crate::websocket;

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

fn get_recipients(target: &Channel) -> Vec<String> {
	match target.channel_type {
		0..=1 => target.recipients.clone().unwrap(),
		_ => vec![]
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
			"type": target.channel_type,
			"recipients": get_recipients(&target),
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
	nonce: String,
}

/// send a message to a channel
#[post("/<target>/messages", data = "<message>")]
pub fn send_message(user: User, target: Channel, message: Json<SendMessage>) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	let content: String = message.content.chars().take(2000).collect();
	let nonce: String = message.nonce.chars().take(32).collect();

	let col = database::get_collection("messages");
	if let Some(_) = col.find_one(doc! { "nonce": nonce.clone() }, None).unwrap() {
		return Some(
			json!({
				"success": false,
				"error": "Message already sent!"
			})
		)
	}

	let id = Ulid::new().to_string();
	Some(if col.insert_one(
		doc! {
			"_id": id.clone(),
			"nonce": nonce.clone(),
			"channel": target.id.clone(),
			"author": user.id.clone(),
			"content": content.clone(),
		},
		None
	).is_ok() {
        if target.channel_type == ChannelType::DM as u8 {
            let col = database::get_collection("channels");
            col.update_one(
                doc! { "_id": target.id.clone() },
                doc! { "$set": { "active": true } },
                None
            ).unwrap();
        }

        websocket::queue_message(
            get_recipients(&target),
            json!({
                "type": "message",
                "data": {
                    "id": id.clone(),
                    "nonce": nonce,
                    "channel": target.id,
                    "author": user.id,
                    "content": content,
                },
            }).to_string()
        );

        json!({
            "success": true,
            "id": id
        })
    } else {
        json!({
            "success": false,
            "error": "Failed database query."
        })
    })
}

/// get a message
#[get("/<target>/messages/<message>")]
pub fn get_message(user: User, target: Channel, message: Message) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
    }
    
    let prev =
        // ! CHECK IF USER HAS PERMISSION TO VIEW EDITS OF MESSAGES
        if let Some(previous) = message.previous_content {
            let mut entries = vec![];
            for entry in previous {
                entries.push(json!({
                    "content": entry.content,
                    "time": entry.time.timestamp(),
                }));
            }

            Some(entries)
        } else {
            None
        };

	Some(
		json!({
			"id": message.id,
			"author": message.author,
			"content": message.content,
            "edited": if let Some(t) = message.edited { Some(t.timestamp()) } else { None },
            "previous_content": prev,
		})
	)
}

#[derive(Serialize, Deserialize)]
pub struct EditMessage {
	content: String,
}

/// edit a message
#[patch("/<target>/messages/<message>", data = "<edit>")]
pub fn edit_message(user: User, target: Channel, message: Message, edit: Json<EditMessage>) -> Option<JsonValue> {
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

            let time =
                if let Some(edited) = message.edited {
                    edited.0
                } else {
                    Ulid::from_string(&message.id).unwrap().datetime()
                };

			let edited = Utc::now();
			match col.update_one(
				doc! { "_id": message.id.clone() },
				doc! {
					"$set": {
						"content": edit.content.clone(),
						"edited": UtcDatetime(edited.clone())
                    },
                    "$push": {
                        "previous_content": {
                            "content": message.content,
                            "time": time,
                        }
                    },
				},
				None
			) {
				Ok(_) => {
					websocket::queue_message(
						get_recipients(&target),
						json!({
							"type": "message_update",
							"data": {
								"id": message.id,
								"channel": target.id,
								"content": edit.content.clone(),
								"edited": edited.timestamp()
							},
						}).to_string()
					);

					json!({
						"success": true
					})
				},
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
				doc! { "_id": message.id.clone() },
				None
			) {
				Ok(_) => {
					websocket::queue_message(
						get_recipients(&target),
						json!({
							"type": "message_delete",
							"data": {
								"id": message.id,
								"channel": target.id
							},
						}).to_string()
					);

					json!({
						"success": true
					})
				},
				Err(_) =>
					json!({
						"success": false,
						"error": "Failed to delete message."
					})
				}
		}
	)
}
