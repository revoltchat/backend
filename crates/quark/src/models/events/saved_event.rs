use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct SavedEvent {
    /// Saved Event ID (composite of user_id:event_id)
    #[serde(rename = "_id")]
    pub id: String,

    /// User ID who saved
    pub user_id: String,

    /// Event ID that was saved
    pub event_id: String,

    /// When the event was saved
    pub created_at: String,
}
