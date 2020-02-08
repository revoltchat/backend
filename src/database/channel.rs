use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    #[serde(rename = "_id")]
    pub id: String,
	pub channel_type: u8,
	
	// for Direct Messages
	pub recipients: Option<Vec<String>>,
	pub active: Option<bool>,
}
