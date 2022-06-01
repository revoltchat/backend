use bson::Document;
use mongodb::options::UpdateOptions;
use ulid::Ulid;

use crate::models::channel_unread::ChannelUnread;
use crate::{AbstractChannelUnread, Error, Result};

use super::super::MongoDb;

static COL: &str = "channel_unreads";

#[async_trait]
impl AbstractChannelUnread for MongoDb {
    async fn acknowledge_message(&self, channel: &str, user: &str, message: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id.channel": channel,
                    "_id.user": user,
                },
                doc! {
                    "$unset": {
                        "mentions": 1_i32
                    },
                    "$set": {
                        "last_id": message
                    }
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel_unread",
            })
    }

    async fn acknowledge_channels(&self, user: &str, channels: &[String]) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id.channel": {
                        "$in": channels
                    },
                    "_id.user": user,
                },
                doc! {
                    "$unset": {
                        "mentions": 1_i32
                    },
                    "$set": {
                        "last_id": Ulid::new().to_string()
                    }
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update",
                with: "channel_unread",
            })
    }

    async fn add_mention_to_unread<'a>(
        &self,
        channel: &str,
        user: &str,
        ids: &[String],
    ) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id.channel": channel,
                    "_id.user": user,
                },
                doc! {
                    "$push": {
                        "mentions": {
                            "$each": ids
                        }
                    }
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel_unread",
            })
    }

    async fn fetch_unreads(&self, user: &str) -> Result<Vec<ChannelUnread>> {
        self.find(
            COL,
            doc! {
                "_id.user": user
            },
        )
        .await
    }
}
