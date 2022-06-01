use serde::{Deserialize, Serialize};

/// Document representing migration information
#[derive(Serialize, Deserialize)]
pub struct MigrationInfo {
    /// Unique Id
    #[serde(rename = "_id")]
    id: i32,
    /// Current database revision
    revision: i32,
}
