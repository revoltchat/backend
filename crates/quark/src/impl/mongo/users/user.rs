use bson::Document;
use futures::StreamExt;
use mongodb::options::{Collation, CollationStrength, FindOneOptions, FindOptions};
use once_cell::sync::Lazy;

use crate::models::user::{FieldsUser, PartialUser, RelationshipStatus, User};
use crate::r#impl::mongo::IntoDocumentPath;
use crate::{AbstractUser, Error, Result};

use super::super::MongoDb;

static FIND_USERNAME_OPTIONS: Lazy<FindOneOptions> = Lazy::new(|| {
    FindOneOptions::builder()
        .collation(
            Collation::builder()
                .locale("en")
                .strength(CollationStrength::Secondary)
                .build(),
        )
        .build()
});

static COL: &str = "users";

#[async_trait]
impl AbstractUser for MongoDb {
    async fn fetch_user(&self, id: &str) -> Result<User> {
        self.find_one_by_id(COL, id).await
    }

    async fn fetch_user_by_username(&self, username: &str, discriminator: &str) -> Result<User> {
        self.find_one_with_options(
            COL,
            doc! {
                "username": username,
                "discriminator": discriminator
            },
            FIND_USERNAME_OPTIONS.clone(),
        )
        .await
    }

    async fn fetch_user_by_token(&self, token: &str) -> Result<User> {
        let session = self
            .col::<Document>("sessions")
            .find_one(
                doc! {
                    "token": token
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "sessions",
            })?
            .ok_or(Error::InvalidSession)?;

        self.fetch_user(session.get_str("user_id").unwrap()).await
    }

    async fn insert_user(&self, user: &User) -> Result<()> {
        self.insert_one(COL, user).await.map(|_| ())
    }

    async fn update_user(
        &self,
        id: &str,
        user: &PartialUser,
        remove: Vec<FieldsUser>,
    ) -> Result<()> {
        self.update_one_by_id(
            COL,
            id,
            user,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        self.delete_one_by_id(COL, id).await.map(|_| ())
    }

    async fn fetch_users<'a>(&self, ids: &'a [String]) -> Result<Vec<User>> {
        let mut cursor = self
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
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "users",
            })?;

        let mut users = vec![];
        while let Some(Ok(user)) = cursor.next().await {
            users.push(user);
        }

        Ok(users)
    }

    async fn fetch_discriminators_in_use(&self, username: &str) -> Result<Vec<String>> {
        Ok(self
            .col::<Document>(COL)
            .find(
                doc! {
                    "username": username
                },
                FindOptions::builder()
                    .collation(
                        Collation::builder()
                            .locale("en")
                            .strength(CollationStrength::Secondary)
                            .build(),
                    )
                    .projection(doc! { "_id": 0, "discriminator": 1 })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "users",
            })?
            .filter_map(|s| async { s.ok() })
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| x.get_str("discriminator").ok().map(|x| x.to_string()))
            .collect::<Vec<String>>())
    }

    async fn fetch_mutual_user_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(self
            .col::<Document>(COL)
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
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "users",
            })?
            .filter_map(|s| async { s.ok() })
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| x.get_str("_id").ok().map(|x| x.to_string()))
            .collect::<Vec<String>>())
    }

    async fn fetch_mutual_channel_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(self
            .col::<Document>("channels")
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
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "channels",
            })?
            .filter_map(|s| async { s.ok() })
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| x.get_str("_id").ok().map(|x| x.to_string()))
            .collect::<Vec<String>>())
    }

    async fn fetch_mutual_server_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(self
            .col::<Document>("server_members")
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
            .map_err(|_| Error::DatabaseError {
                operation: "aggregate",
                with: "server_members",
            })?
            .filter_map(|s| async { s.ok() })
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| x.get_str("_id").ok().map(|i| i.to_string()))
            .collect::<Vec<String>>())
    }

    async fn set_relationship(
        &self,
        user_id: &str,
        target_id: &str,
        relationship: &RelationshipStatus,
    ) -> Result<()> {
        if let RelationshipStatus::None = relationship {
            return self.pull_relationship(user_id, target_id).await;
        }

        self.col::<Document>(COL)
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
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })
    }

    async fn pull_relationship(&self, user_id: &str, target_id: &str) -> Result<()> {
        self.col::<Document>(COL)
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
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "user",
            })
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
            FieldsUser::DisplayName => "display_name",
        })
    }
}
