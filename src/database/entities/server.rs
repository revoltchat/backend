use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemberCompositeKey {
    pub guild: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub owner: String,

    pub channels: Vec<String>,
    pub invites: Vec<Invite>,
    pub bans: Vec<Ban>,

    pub default_permissions: u32,
}
