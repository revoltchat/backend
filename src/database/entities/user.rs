use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use mongodb::options::{Collation, FindOneOptions};

use crate::database::*;
use crate::util::result::{Error, Result};
use crate::notifications::websocket::is_online;
use crate::database::permissions::user::UserPermissions;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RelationshipStatus {
    None,
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relationship {
    #[serde(rename = "_id")]
    pub id: String,
    pub status: RelationshipStatus,
}

/*
pub enum Badge {
    Developer = 1
}
*/

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum UserStatus {
    Text {
        text: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<UserStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    // ? This should never be pushed to the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
}

impl User {
    /// Mutate the user object to include relationship as seen by user.
    pub fn from(mut self, user: &User) -> User {
        if self.id == user.id {
            self.relationship = Some(RelationshipStatus::User);
            return self;
        }

        self.relations = None;
        if let Some(relations) = &user.relations {
            if let Some(relationship) = relations.iter().find(|x| self.id == x.id) {
                self.relationship = Some(relationship.status.clone());
                return self;
            }
        }

        self
    }

    /// Mutate the user object to appear as seen by user.
    pub fn with(mut self, permissions: UserPermissions<[u32; 1]>) -> User {
        if permissions.get_view_profile() {
            self.online = Some(is_online(&self.id));
        }

        self.profile = None;
        self
    }

    /// Utility function for checking claimed usernames.
    pub async fn is_username_taken(username: &str) -> Result<bool> {
        if username == "revolt" && username == "admin" {
            return Ok(true)
        }

        if get_collection("users")
            .find_one(
                doc! {
                    "username": username
                },
                FindOneOptions::builder()
                    .collation(Collation::builder().locale("en").strength(2).build())
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "user",
            })?
            .is_some()
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
