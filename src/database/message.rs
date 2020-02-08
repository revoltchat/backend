use serde::{ Deserialize, Serialize };
use bson::{ UtcDateTime };

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(rename = "_id")]
	pub id: String,
	pub channel: String,
	pub author: String,

	pub content: String,
	pub edited: Option<UtcDateTime>,
}
