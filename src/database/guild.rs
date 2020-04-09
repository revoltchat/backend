use bson::doc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Member {
    pub id: String,
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Invite {
    pub id: String,
    pub custom: bool,
    pub channel: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Guild {
    #[serde(rename = "_id")]
    pub id: String,
    // pub nonce: String, used internally
    pub name: String,
    pub description: String,
    pub owner: String,

    pub channels: Vec<String>,
    pub members: Vec<Member>,
    pub invites: Vec<Invite>,

    pub default_permissions: u32,
}
