use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};

use crate::models::attachment::File;

/// Composite primary key consisting of server and user id
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
pub struct MemberCompositeKey {
    /// Server Id
    pub server: String,
    /// User Id
    pub user: String,
}

/// Representation of a member of a server on Revolt
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialMember"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Member {
    /// Unique member id
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,

    /// Time at which this user joined the server
    pub joined_at: Timestamp,

    /// Member's nickname
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// Avatar attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    /// Member's roles
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roles: Vec<String>,
    /// Timestamp this member is timed out until
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Timestamp>,
}

/// Optional fields on server member object
#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Clone)]
pub enum FieldsMember {
    Nickname,
    Avatar,
    Roles,
    Timeout,
}

/// Member removal intention
pub enum RemovalIntention {
    Leave,
    Kick,
    Ban,
}
