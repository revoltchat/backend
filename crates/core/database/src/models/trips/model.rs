use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::util::iso_bson_chrono; // Adjust path as needed

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trip {
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub destination: String,

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[serde(
        serialize_with = "iso_bson_chrono::serialize",
        deserialize_with = "iso_bson_chrono::deserialize"
    )]
    pub start_date: DateTime<Utc>,

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[serde(
        serialize_with = "iso_bson_chrono::serialize",
        deserialize_with = "iso_bson_chrono::deserialize"
    )]
    pub end_date: DateTime<Utc>,

    pub purpose: String,
    pub user_id: String,
}
