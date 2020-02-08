use crate::database::{ user::User, channel::Channel };

use rocket_contrib::json::{ JsonValue };
use num_enum::TryFromPrimitive;
use bson::{ doc };

#[derive(Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum ChannelType {
	DM = 0,
	GROUP_DM = 1,
	GUILD_CHANNEL = 2,
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
