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
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialMember"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Member {
    /// Unique member id
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,

    /// Member's nickname
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// Avatar attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    /// Member's roles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

/// Optional fields on server member object
#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Clone)]
pub enum FieldsMember {
    Nickname,
    Avatar,
    Roles,
}

/// Member removal intention
pub enum RemovalIntention {
    Leave,
    Kick,
    Ban,
}
