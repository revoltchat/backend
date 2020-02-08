use crate::guards::{ auth::User, channel::Channel };
use crate::database;

use rocket_contrib::json::{ Json, JsonValue };
use serde::{ Serialize, Deserialize };
use mongodb::options::FindOptions;
use num_enum::TryFromPrimitive;
use bson::{ bson, doc };
use ulid::Ulid;

#[derive(Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum ChannelType {
	DM = 0,
	GROUP_DM = 1,
	GUILD_CHANNEL = 2,
}

fn has_permission(user: &User, target: &Channel) -> bool {
	let id = user.0.to_string();
	match target.1 {
		ChannelType::DM |
		ChannelType::GROUP_DM => {
			for user in target.2.get_array("recipients").expect("DB[recipients]") {
				if user.as_str().expect("Expected string id.") == id {
					return true;
				}
			}

			false
		},
		ChannelType::GUILD_CHANNEL =>
			false
	}
}

/// fetch channel information
#[get("/<target>")]
pub fn channel(user: User, target: Channel) -> Option<JsonValue> {
	if !has_permission(&user, &target) {
		return None
	}

	let Channel ( id, channel_type, doc ) = target;

	Some(
		json!({
			"id": id.to_string(),
			"type": channel_type as u8
		}
	))
}
