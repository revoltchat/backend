use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    pub id: String,
    pub status: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: Option<String>,
    pub relations: Option<Vec<Relationship>>,
}
