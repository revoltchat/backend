use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriendStatus {
    pub id: String,
    pub user: String,
    pub status: i32,
}
