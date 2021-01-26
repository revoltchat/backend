use serde::{Deserialize, Serialize};

use crate::{database::permissions::user::UserPermissions, notifications::websocket::is_online};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,

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
        if !permissions.get_view_all() {
            self.relations = None;
        }

        if permissions.get_view_profile() {
            self.online = Some(is_online(&self.id));
        }

        self
    }
}
