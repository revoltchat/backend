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
}
