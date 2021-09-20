use futures::StreamExt;
use mongodb::bson::Document;
use mongodb::options::{Collation, FindOneOptions};
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::ops;
use ulid::Ulid;
use validator::Validate;

use crate::database::permissions::user::UserPermissions;
use crate::database::*;
use crate::notifications::websocket::is_online;
use crate::util::result::{Error, Result};
use crate::util::variables::EARLY_ADOPTER_BADGE;
use crate::util::variables::MAX_SERVER_COUNT;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    #[serde(rename = "_id")]
    pub id: String,
    pub status: RelationshipStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Presence {
    Online,
    Idle,
    Busy,
    Invisible,
}

#[derive(Validate, Serialize, Deserialize, Debug, Clone)]
pub struct UserStatus {
    #[validate(length(min = 1, max = 128))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<File>,
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(i32)]
pub enum Badges {
    Developer = 1,
    Translator = 2,
    Supporter = 4,
    ResponsibleDisclosure = 8,
    RevoltTeam = 16,
    EarlyAdopter = 256,
}

impl_op_ex_commutative!(+ |a: &i32, b: &Badges| -> i32 { *a | *b as i32 });

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotInformation {
    owner: String
}

// When changing this struct, update notifications/payload.rs#113
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfile>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInformation>,

    // ? This should never be pushed to the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
}

impl User {
    /// Mutate the user object to include relationship as seen by user.
    pub fn from(mut self, user: &User) -> User {
        self.relationship = Some(RelationshipStatus::None);

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

    /// Apply any relevant badges.
    pub fn apply_badges(mut self) -> User {
        let mut badges = self.badges.unwrap_or_else(|| 0);
        if let Ok(id) = Ulid::from_string(&self.id) {
            if id.datetime().timestamp_millis() < *EARLY_ADOPTER_BADGE {
                badges = badges + Badges::EarlyAdopter;
            }
        }

        self.badges = Some(badges);
        self
    }

    /// Mutate the user object to appear as seen by user.
    pub fn with(self, permissions: UserPermissions<[u32; 1]>) -> User {
        let mut user = self.apply_badges();

        if permissions.get_view_profile() {
            user.online = Some(is_online(&user.id));
        } else {
            user.status = None;
        }

        // If the user's status is `Presence::Invisible`, return it as `Presence::Offline`
        if let Some(status) = &user.status {
            if let Some(presence) = &status.presence {
                if presence == &Presence::Invisible {
                    user.status = None;
                    user.online = Some(false);
                }
            }
        }

        user.profile = None;
        user
    }

    /// Mutate the user object to appear as seen by user.
    /// Also overrides the relationship status.
    pub async fn from_override(
        mut self,
        user: &User,
        relationship: RelationshipStatus,
    ) -> Result<User> {
        let permissions = PermissionCalculator::new(&user)
            .with_relationship(&relationship)
            .for_user(&self.id)
            .await?;

        self.relations = None;
        self.relationship = Some(relationship);
        Ok(self.with(permissions))
    }

    /// Utility function for checking claimed usernames.
    pub async fn is_username_taken(username: &str) -> Result<bool> {
        if username.to_lowercase() == "revolt" || username.to_lowercase() == "admin" || username.to_lowercase() == "system" {
            return Ok(true);
        }
        match db_conn().get_user_by_username(username).await {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(e) => Err(e)
        }
    }

    /// Utility function for fetching multiple users from the perspective of one.
    /// Assumes user has a mutual connection with others.
    pub async fn fetch_multiple_users(&self, user_ids: Vec<String>) -> Result<Vec<User>> {
        let other_users = db_conn().get_users(&user_ids).await?;
        let mut users = vec![];
        for other in other_users {
            let permissions = PermissionCalculator::new(&self)
                .with_mutual_connection()
                .with_user(&other)
                .for_user_given()
                .await?;
            users.push(other.from(&self).with(permissions));
        }
        Ok(users)
    }

    /// Utility function to get all of a user's memberships.
    pub async fn fetch_memberships(id: &str) -> Result<Vec<Member>> {
        db_conn().get_users_memberships(id).await
    }

    /// Utility function to get all the server IDs the user is in.
    pub async fn fetch_server_ids(id: &str) -> Result<Vec<String>> {
        let memberships = db_conn().get_users_memberships(id).await?;
        Ok(memberships.iter().map(|e| e.id.server.to_string()).collect())
    }

    /// Utility function to fetch unread objects for user.
    pub async fn fetch_unreads(id: &str) -> Result<Vec<Document>> {
        db_conn().get_unreads_for_user(id).await
    }

    /// Check if this user can acquire another server.
    pub async fn can_acquire_server(id: &str) -> Result<bool> {
        let server_ids = User::fetch_server_ids(&id).await?;
        Ok(server_ids.len() < *MAX_SERVER_COUNT)
    }
}
