use crate::models::File;
use serde::{Deserialize, Serialize};

/// Respresents a webhook
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialWebhook"]
#[opt_skip_serializing_none]
#[opt_some_priority]

pub struct Webhook {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,

    /// The name of the webhook
    pub name: String,

    /// The avatar of the webhook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    /// The channel this webhook belongs to
    pub channel: String,

    /// The private token for the webhook
    pub token: String
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum FieldsWebhook {
    Avatar
}
