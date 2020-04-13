use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserJoin {
    pub id: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLeave {
    pub id: String,
    pub user: String,
    pub banned: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelCreate {
    pub id: String,
    pub channel: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelDelete {
    pub id: String,
    pub channel: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Delete {
    pub id: String,
}
