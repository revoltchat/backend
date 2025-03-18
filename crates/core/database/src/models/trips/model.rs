use crate::util::iso_bson_chrono;
#[cfg(feature = "mongodb")]
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize}; // Adjust path as needed

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trip {
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,

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

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub purpose: String,

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub user_id: String,

    #[cfg_attr(feature = "schemars", schemars(with = "Option<String>"))]
    pub description: Option<String>,

    #[cfg_attr(feature = "schemars", schemars(with = "Option<String>"))]
    #[serde(
        skip_serializing,
        skip_deserializing,
        serialize_with = "iso_bson_chrono::serialize_optional",
        deserialize_with = "iso_bson_chrono::deserialize_optional"
    )]
    pub deletion_date: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TripComment {
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub trip_id: ObjectId,

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub user_id: String,

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    pub content: String,

    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
    #[serde(
        serialize_with = "iso_bson_chrono::serialize",
        deserialize_with = "iso_bson_chrono::deserialize"
    )]
    pub created_at: DateTime<Utc>,
}
