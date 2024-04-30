use serde::{Deserialize, Serialize};

/// Simple database model for testing
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleModel {
    pub number: i32,
    pub value: String,
}
