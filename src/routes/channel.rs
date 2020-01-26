use crate::guards::auth::User;
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
