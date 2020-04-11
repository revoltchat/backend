use bson::doc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MemberRef {
    pub guild: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberRef,
    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Invite {
    pub code: String,
    pub creator: String,
    pub channel: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ban {
    pub id: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Guild {
    #[serde(rename = "_id")]
    pub id: String,
    // pub nonce: String, used internally
    pub name: String,
    pub description: String,
    pub owner: String,

    pub invites: Vec<Invite>,
    pub bans: Vec<Ban>,

    pub default_permissions: u32,
}
