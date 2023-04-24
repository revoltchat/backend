use crate::Override;
use serde::{Deserialize, Serialize};

/// Data permissions Field - contains both allow and deny
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, Copy)]
pub struct DataPermissionsField {
    pub permissions: Override,
}

/// Data permissions Value - contains allow
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, Copy)]
pub struct DataPermissionsValue {
    pub permissions: u64,
}

/// Data permissions Poly - can contain either Value or Field
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum DataPermissionPoly {
    Value {
        /// Permission values to set for members in a `Group`
        permissions: u64,
    },
    Field {
        /// Allow / deny values to set for members in this `TextChannel` or `VoiceChannel`
        permissions: Override,
    },
}
