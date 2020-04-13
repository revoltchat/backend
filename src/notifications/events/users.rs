use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriendStatus {
    pub id: String,
    pub status: i32,
}
