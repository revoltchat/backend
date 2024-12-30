use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialUserWhiteList"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct UserWhiteList {
    /// User id of the owner
    pub email: String,

    /// Name user white listed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Description for the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}
