use bson::Document;
use mongodb::options::UpdateOptions;
use revolt_result::Result;
use ulid::Ulid;

use crate::ChannelUnread;
use crate::MongoDb;

use super::AbstractChannelUnreads;

static COL: &str = "channel_unreads";

#[async_trait]
impl AbstractChannelUnreads for MongoDb {
    /// Acknowledge a message.
    async fn acknowledge_message(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id.channel": channel_id,
                    "_id.user": user_id,
                },
                doc! {
                    "$unset": {
                        "mentions": 1_i32
                    },
                    "$set": {
                        "last_id": message_id
                    }
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", COL))
    }

    /// Acknowledge many channels.
    async fn acknowledge_channels(&self, user_id: &str, channel_ids: &[String]) -> Result<()> {
        let current_time = Ulid::new().to_string();

        self.col::<Document>(COL)
            .delete_many(
                doc! {
                    "_id.channel": {
                        "$in": channel_ids
                    },
                    "_id.user": user_id
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("delete_many", COL))?;

        self.col::<Document>(COL)
            .insert_many(
                channel_ids
                    .iter()
                    .map(|channel_id| {
                        doc! {
                            "_id": {
                                "channel": channel_id,
                                "user": user_id
                            },
                            "last_id": &current_time
                        }
                    })
                    .collect::<Vec<Document>>(),
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_many", COL))
    }

    /// Add a mention.
    async fn add_mention_to_unread<'a>(
        &self,
        channel_id: &str,
        user_id: &str,
        message_ids: &[String],
    ) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id.channel": channel_id,
                    "_id.user": user_id,
                },
                doc! {
                    "$push": {
                        "mentions": {
                            "$each": message_ids
                        }
                    }
                },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", COL))
    }

    /// Fetch all channel unreads for a user.
    async fn fetch_unreads(&self, user_id: &str) -> Result<Vec<ChannelUnread>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "_id.user": user_id
            }
        )
    }
}
