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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    /// Utility function for fetching multiple users from the perspective of one.
    /// Assumes user has a mutual connection with others.
    pub async fn fetch_multiple_users(&self, user_ids: Vec<String>) -> Result<Vec<User>> {
        let mut users = vec![];
        let mut cursor = get_collection("users")
            .find(
                doc! {
                    "_id": {
                        "$in": user_ids
                    }
                },
                FindOptions::builder()
                    .projection(
                        doc! { "_id": 1, "username": 1, "avatar": 1, "badges": 1, "status": 1, "flags": 1, "bot": 1 },
                    )
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "users",
            })?;

        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                let other: User = from_document(doc).map_err(|_| Error::DatabaseError {
                    operation: "from_document",
                    with: "user",
                })?;

                let permissions = PermissionCalculator::new(&self)
                    .with_mutual_connection()
                    .with_user(&other)
                    .for_user_given()
                    .await?;

                users.push(other.from(&self).with(permissions));
            }
        }

        Ok(users)
    }

    /// Utility function to get all of a user's memberships.
    pub async fn fetch_memberships(id: &str) -> Result<Vec<Member>> {
        Ok(get_collection("server_members")
            .find(
                doc! {
                    "_id.user": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "server_members",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| {
                from_document(x).ok()
            })
            .collect::<Vec<Member>>())
    }

    /// Utility function to get all the server IDs the user is in.
    pub async fn fetch_server_ids(id: &str) -> Result<Vec<String>> {
        Ok(get_collection("server_members")
            .find(
                doc! {
                    "_id.user": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "server_members",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| {
                x.get_document("_id")
                    .ok()
                    .map(|i| i.get_str("server").ok().map(|x| x.to_string()))
            })
            .flatten()
            .collect::<Vec<String>>())
    }

    /// Utility function to fetch unread objects for user.
    pub async fn fetch_unreads(id: &str) -> Result<Vec<Document>> {
        Ok(get_collection("channel_unreads")
            .find(
                doc! {
                    "_id.user": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "user_settings",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await)
    }
}
