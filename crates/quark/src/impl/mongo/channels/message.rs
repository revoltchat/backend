use bson::{to_bson, Document};
use futures::try_join;
use mongodb::options::FindOptions;

use crate::models::message::{AppendMessage, Message, MessageSort, PartialMessage};
use crate::r#impl::mongo::DocumentId;
use crate::{AbstractMessage, Error, Result};

use super::super::MongoDb;

static COL: &str = "messages";

impl MongoDb {
    pub async fn delete_bulk_messages(&self, projection: Document) -> Result<()> {
        let mut for_attachments = projection.clone();
        for_attachments.insert(
            "attachments",
            doc! {
                "$exists": 1_i32
            },
        );

        // Check if there are any attachments we need to delete.
        let message_ids_with_attachments = self
            .find_with_options::<_, DocumentId>(
                COL,
                for_attachments,
                FindOptions::builder()
                    .projection(doc! { "_id": 1_i32 })
                    .build(),
            )
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<String>>();

        // If we found any, mark them as deleted.
        if !message_ids_with_attachments.is_empty() {
            self.col::<Document>("attachments")
                .update_many(
                    doc! {
                        "message_id": {
                            "$in": message_ids_with_attachments
                        }
                    },
                    doc! {
                        "$set": {
                            "deleted": true
                        }
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_many",
                    with: "attachments",
                })?;
        }

        // And then delete said messages.
        self.col::<Document>(COL)
            .delete_many(projection, None)
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "messages",
            })
    }
}

#[async_trait]
impl AbstractMessage for MongoDb {
    async fn fetch_message(&self, id: &str) -> Result<Message> {
        self.find_one_by_id(COL, id).await
    }

    async fn insert_message(&self, message: &Message) -> Result<()> {
        self.insert_one(COL, message).await.map(|_| ())
    }

    async fn update_message(&self, id: &str, message: &PartialMessage) -> Result<()> {
        self.update_one_by_id(COL, id, message, vec![], None)
            .await
            .map(|_| ())
    }

    async fn append_message(&self, id: &str, append: &AppendMessage) -> Result<()> {
        let mut query = doc! {};

        if let Some(embeds) = &append.embeds {
            if !embeds.is_empty() {
                query.insert(
                    "$push",
                    doc! {
                        "embeds": {
                            "$each": to_bson(embeds)
                                .map_err(|_| Error::DatabaseError {
                                    operation: "to_bson",
                                    with: "embeds"
                                })?
                        }
                    },
                );
            }
        }

        if query.is_empty() {
            return Ok(());
        }

        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                query,
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "message",
            })
            .map(|_| ())
    }

    async fn delete_message(&self, id: &str) -> Result<()> {
        self.delete_one_by_id(COL, id).await.map(|_| ())
    }

    async fn delete_messages(&self, channel: &str, ids: Vec<String>) -> Result<()> {
        self.delete_bulk_messages(doc! {
            "channel": channel,
            "_id": {
                "$in": ids
            }
        })
        .await
    }

    async fn fetch_messages(
        &self,
        channel: &str,
        limit: Option<i64>,
        before: Option<String>,
        after: Option<String>,
        sort: Option<MessageSort>,
        nearby: Option<String>,
    ) -> Result<Vec<Message>> {
        let limit = limit.unwrap_or(50);
        Ok(if let Some(nearby) = nearby {
            let (a, b) = try_join!(
                self.find_with_options::<_, Message>(
                    COL,
                    doc! {
                        "channel": channel,
                        "_id": {
                            "$gte": &nearby
                        }
                    },
                    FindOptions::builder()
                        .limit(limit / 2 + 1)
                        .sort(doc! {
                            "_id": 1_i32
                        })
                        .build(),
                ),
                self.find_with_options::<_, Message>(
                    COL,
                    doc! {
                        "channel": channel,
                        "_id": {
                            "$lt": &nearby
                        }
                    },
                    FindOptions::builder()
                        .limit(limit / 2)
                        .sort(doc! {
                            "_id": -1_i32
                        })
                        .build(),
                )
            )?;

            [a, b].concat()
        } else {
            let mut query = doc! { "channel": channel };
            if let Some(before) = before {
                query.insert("_id", doc! { "$lt": before });
            }

            if let Some(after) = after {
                query.insert("_id", doc! { "$gt": after });
            }

            let sort: i32 = if let MessageSort::Latest = sort.unwrap_or(MessageSort::Latest) {
                -1
            } else {
                1
            };

            self.find_with_options::<_, Message>(
                COL,
                query,
                FindOptions::builder()
                    .limit(limit)
                    .sort(doc! {
                        "_id": sort
                    })
                    .build(),
            )
            .await?
        })
    }

    async fn search_messages(
        &self,
        channel: &str,
        query: &str,
        limit: Option<i64>,
        before: Option<String>,
        after: Option<String>,
        sort: MessageSort,
    ) -> Result<Vec<Message>> {
        let limit = limit.unwrap_or(50);

        let mut filter = doc! {
            "channel": channel,
            "$text": {
                "$search": query
            }
        };

        if let Some(doc) = match (before, after) {
            (Some(before), Some(after)) => Some(doc! {
                "lt": before,
                "gt": after
            }),
            (Some(before), _) => Some(doc! {
                "lt": before
            }),
            (_, Some(after)) => Some(doc! {
                "gt": after
            }),
            _ => None,
        } {
            filter.insert("_id", doc);
        }

        self.find_with_options(
            COL,
            filter,
            FindOptions::builder()
                .projection(if let MessageSort::Relevance = &sort {
                    doc! {
                        "score": {
                            "$meta": "textScore"
                        }
                    }
                } else {
                    doc! {}
                })
                .limit(limit)
                .sort(match &sort {
                    MessageSort::Relevance => doc! {
                        "score": {
                            "$meta": "textScore"
                        }
                    },
                    MessageSort::Latest => doc! {
                        "_id": -1_i32
                    },
                    MessageSort::Oldest => doc! {
                        "_id": 1_i32
                    },
                })
                .build(),
        )
        .await
    }

    /// Add a new reaction to a message
    async fn add_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$addToSet": {
                        format!("reactions.{emoji}"): user
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "message",
            })
    }

    /// Remove a reaction from a message
    async fn remove_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$pull": {
                        format!("reactions.{emoji}"): user
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "message",
            })
    }

    /// Remove reaction from a message
    async fn clear_reaction(&self, id: &str, emoji: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$unset": {
                        format!("reactions.{emoji}"): 1
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "message",
            })
    }
}
