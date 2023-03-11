use bson::{to_bson, Document};
use futures::try_join;
use mongodb::options::FindOptions;

use crate::models::message::{
    AppendMessage, Message, MessageQuery, MessageSort, MessageTimePeriod, PartialMessage,
};
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

    async fn fetch_messages(&self, query: MessageQuery) -> Result<Vec<Message>> {
        let mut filter = doc! {};

        // 1. Apply message filters
        if let Some(channel) = query.filter.channel {
            filter.insert("channel", channel);
        }

        if let Some(author) = query.filter.author {
            filter.insert("author", author);
        }

        let is_search_query = if let Some(query) = query.filter.query {
            filter.insert(
                "$text",
                doc! {
                    "$search": query
                },
            );

            true
        } else {
            false
        };

        // 2. Find query limit
        let limit = query.limit.unwrap_or(50);

        // 3. Apply message time period
        match query.time_period {
            MessageTimePeriod::Relative { nearby } => {
                // 3.1. Prepare filters
                let mut older_message_filter = filter.clone();
                let mut newer_message_filter = filter;

                older_message_filter.insert(
                    "_id",
                    doc! {
                        "$lt": &nearby
                    },
                );

                newer_message_filter.insert(
                    "_id",
                    doc! {
                        "$gte": &nearby
                    },
                );

                // 3.2. Execute in both directions
                let (a, b) = try_join!(
                    self.find_with_options::<_, Message>(
                        COL,
                        newer_message_filter,
                        FindOptions::builder()
                            .limit(limit / 2 + 1)
                            .sort(doc! {
                                "_id": 1_i32
                            })
                            .build(),
                    ),
                    self.find_with_options::<_, Message>(
                        COL,
                        older_message_filter,
                        FindOptions::builder()
                            .limit(limit / 2)
                            .sort(doc! {
                                "_id": -1_i32
                            })
                            .build(),
                    )
                )?;

                Ok([a, b].concat())
            }
            MessageTimePeriod::Absolute {
                before,
                after,
                sort,
            } => {
                // 3.1. Apply message ID filter
                if let Some(doc) = match (before, after) {
                    (Some(before), Some(after)) => Some(doc! {
                        "$lt": before,
                        "$gt": after
                    }),
                    (Some(before), _) => Some(doc! {
                        "$lt": before
                    }),
                    (_, Some(after)) => Some(doc! {
                        "$gt": after
                    }),
                    _ => None,
                } {
                    filter.insert("_id", doc);
                }

                // 3.2. Execute with given message sort
                self.find_with_options(
                    COL,
                    filter,
                    FindOptions::builder()
                        .limit(limit)
                        .sort(match sort.unwrap_or(MessageSort::Latest) {
                            // Sort by relevance, fallback to latest
                            MessageSort::Relevance => {
                                if is_search_query {
                                    doc! {
                                        "score": {
                                            "$meta": "textScore"
                                        }
                                    }
                                } else {
                                    doc! {
                                        "_id": -1_i32
                                    }
                                }
                            }
                            // Sort by latest first
                            MessageSort::Latest => doc! {
                                "_id": -1_i32
                            },
                            // Sort by oldest first
                            MessageSort::Oldest => doc! {
                                "_id": 1_i32
                            },
                        })
                        .build(),
                )
                .await
            }
        }
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
