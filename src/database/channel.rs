use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
	pub channel_type: u8,

	pub last_message: Option<String>,
	
	// for Direct Messages
	pub recipients: Option<Vec<String>>,
	pub active: Option<bool>,
}
