use ::mongodb::options::{Collation, CollationStrength, FindOneOptions, FindOptions};
use authifier::models::Session;
use futures::StreamExt;
use revolt_result::Result;

use crate::DocumentId;
use crate::IntoDocumentPath;
use crate::MongoDb;
use crate::{FieldsUser, PartialUser, RelationshipStatus, User};

use super::AbstractUsers;

static COL: &str = "users";

#[async_trait]
impl AbstractUsers for MongoDb {
    /// Insert a new user into the database
    async fn insert_user(&self, user: &User) -> Result<()> {
        query!(self, insert_one, COL, &user).map(|_| ())
    }

    /// Fetch a user from the database
    async fn fetch_user(&self, id: &str) -> Result<User> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a user from the database by their username
    async fn fetch_user_by_username(&self, username: &str) -> Result<User> {
        query!(
            self,
            find_one_with_options,
            COL,
            doc! {
                "username": username
            },
            FindOneOptions::builder()
                .collation(
                    Collation::builder()
                        .locale("en")
                        .strength(CollationStrength::Secondary)
                        .build(),
                )
                .build()
        )?
        .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a user from the database by their session token
    async fn fetch_user_by_token(&self, token: &str) -> Result<User> {
        let session = self
            .col::<Session>("sessions")
            .find_one(
                doc! {
                    "token": token
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find_one", "sessions"))?
            .ok_or_else(|| create_error!(InvalidSession))?;

        self.fetch_user(&session.id).await
    }

    /// Fetch multiple users by their ids
    async fn fetch_users<'a>(&self, ids: &'a [String]) -> Result<Vec<User>> {
        Ok(self
            .col::<User>(COL)
            .find(
                doc! {
                    "_id": {
                        "$in": ids
                    }
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("find", COL))?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }

    /// Fetch ids of users that both users are friends with
    async fn fetch_mutual_user_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(self
            .col::<DocumentId>(COL)
            .find(
                doc! {
                    "$and": [
                        { "relations": { "$elemMatch": { "_id": &user_a, "status": "Friend" } } },
                        { "relations": { "$elemMatch": { "_id": &user_b, "status": "Friend" } } }
                    ]
                },
                FindOptions::builder().projection(doc! { "_id": 1 }).build(),
            )
            .await
            .map_err(|_| create_database_error!("find", COL))?
            .filter_map(|s| async { s.ok() })
            .map(|user| user.id)
            .collect()
            .await)
    }

    /// Fetch ids of channels that both users are in
    async fn fetch_mutual_channel_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(self
            .col::<DocumentId>("channels")
            .find(
                doc! {
                    "channel_type": {
                        "$in": ["Group", "DirectMessage"]
                    },
                    "recipients": {
                        "$all": [ user_a, user_b ]
                    }
                },
                FindOptions::builder().projection(doc! { "_id": 1 }).build(),
            )
            .await
            .map_err(|_| create_database_error!("find", "channels"))?
            .filter_map(|s| async { s.ok() })
            .map(|user| user.id)
            .collect()
            .await)
    }

    /// Fetch ids of servers that both users share
    async fn fetch_mutual_server_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(self
            .col::<DocumentId>("server_members")
            .aggregate(
                vec![
                    doc! {
                        "$match": {
                            "_id.user": user_a
                        }
                    },
                    doc! {
                        "$lookup": {
                            "from": "server_members",
                            "as": "members",
                            "let": {
                                "server": "$_id.server"
                            },
                            "pipeline": [
                                {
                                    "$match": {
                                        "$expr": {
                                            "$and": [
                                                { "$eq": [ "$_id.user", user_b ] },
                                                { "$eq": [ "$_id.server", "$$server" ] }
                                            ]
                                        }
                                    }
                                }
                            ]
                        }
                    },
                    doc! {
                        "$match": {
                            "members": {
                                "$size": 1_i32
                            }
                        }
                    },
                    doc! {
                        "$project": {
                            "_id": "$_id.server"
                        }
                    },
                ],
                None,
            )
            .await
            .map_err(|_| create_database_error!("aggregate", "server_members"))?
            .filter_map(|s| async { s.ok() })
            .filter_map(|doc| async move { doc.get_str("_id").map(|id| id.to_string()).ok() })
            .collect()
            .await)
    }

    /// Update a user by their id given some data
    async fn update_user(
        &self,
        id: &str,
        partial: &PartialUser,
        remove: Vec<FieldsUser>,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            id,
            partial,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None
        )
        .map(|_| ())
    }

    /// Set relationship with another user
    ///
    /// This should use pull_relationship if relationship is None.
    async fn set_relationship(
        &self,
        user_id: &str,
        target_id: &str,
        relationship: &RelationshipStatus,
    ) -> Result<()> {
        if let RelationshipStatus::None = relationship {
            return self.pull_relationship(user_id, target_id).await;
        }

        self.col::<User>(COL)
            .update_one(
                doc! {
                    "_id": user_id
                },
                vec![doc! {
                    "$set": {
                        "relations": {
                            "$concatArrays": [
                                {
                                    "$ifNull": [
                                        {
                                            "$filter": {
                                                "input": "$relations",
                                                "cond": {
                                                    "$ne": [
                                                        "$$this._id",
                                                        target_id
                                                    ]
                                                }
                                            }
                                        },
                                        []
                                    ]
                                },
                                [
                                    {
                                        "_id": target_id,
                                        "status": format!("{relationship:?}")
                                    }
                                ]
                            ]
                        }
                    }
                }],
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", "user"))
    }

    /// Remove relationship with another user
    async fn pull_relationship(&self, user_id: &str, target_id: &str) -> Result<()> {
        self.col::<User>(COL)
            .update_one(
                doc! {
                    "_id": user_id
                },
                doc! {
                    "$pull": {
                        "relations": {
                            "_id": target_id
                        }
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", COL))
    }

    /// Delete a user by their id
    async fn delete_user(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }
}

impl IntoDocumentPath for FieldsUser {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsUser::Avatar => "avatar",
            FieldsUser::ProfileBackground => "profile.background",
            FieldsUser::ProfileContent => "profile.content",
            FieldsUser::StatusPresence => "status.presence",
            FieldsUser::StatusText => "status.text",
            FieldsUser::Pronouns => "pronouns",
        })
    }
}
