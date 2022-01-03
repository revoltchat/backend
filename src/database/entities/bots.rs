use serde::{Serialize, Deserialize};

pub fn if_false(t: &bool) -> bool {
    !t
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bot {
    #[serde(rename = "_id")]
    pub id: String,
    pub owner: String,
    pub token: String,
    pub public: bool,
    #[serde(skip_serializing_if = "if_false", default)]
    pub analytics: bool,
    #[serde(skip_serializing_if = "if_false", default)]
    pub discoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactions_url: Option<String>,
}
