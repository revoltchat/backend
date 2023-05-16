use super::AbstractChannels;
use crate::{Channel, FieldsChannel, IntoDocumentPath, MongoDb, PartialChannel};
use bson::Document;
use futures::{FutureExt, StreamExt};
use revolt_permissions::OverrideField;
use revolt_result::Result;
static COL: &str = "channels";

#[async_trait]
impl AbstractChannels for MongoDb {
    /// Insert a new channel in the database
    async fn insert_channel(&self, channel: &Channel) -> Result<()> {
        query!(self, insert_one, COL, &channel).map(|_| ())
    }

    /// Insert a a user to a group
    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()> {
        let user_doc = doc! {
            "_id": channel
        };
        let group_doc = doc! {
            "$push": {
                "recipients": user
            }
        };
        self.col::<Document>(COL)
            .update_one(user_doc, group_doc, None)
            .map(|_| ())
            .await;
        Ok(())
    }

    /// Insert channel role permissions
    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        let channel_doc = doc! { "_id": channel };
        let role_doc = doc! {
            "$set": {
                "role_permissions.".to_owned() + role: permissions
            }
        };

        self.col::<Document>(COL)
            .update_one(channel_doc, role_doc, None)
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", "channel"))
    }

    /// Fetch a channel from the database
    async fn fetch_channel(&self, id: &str) -> Result<Channel> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all channels from the database
    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>> {
        Ok(self
            .col::<Channel>(COL)
            .find(
                doc! {
                    "_id": {
                        "$in": ids
                    }
                },
                None,
            )
            .await
            .map_err(|_| create_database_error!("fetch", "channels"))?
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

    /// Fetch all direct messages for a user
    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>> {
        let doc = doc! {
            "$or": [
                    {
                        "$or": [
                            {
                                "channel_type": "DirectMessage"
                            },
                            {
                                "channel_type": "Group"
                            }
                        ],
                        "recipients": user_id
                    },
                    {
                        "channel_type": "SavedMessages",
                        "user": user_id
                    }
                ]
        };

        query!(self, find, COL, doc)
    }

    // Fetch saved messages channel
    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel> {
        let doc = doc! {
            "channel_type": "SavedMessages",
                "user": user_id
        };
        query!(self, find_one, COL, doc)?.ok_or_else(|| create_error!(InternalError))
    }

    // Fetch direct message channel (PMs)
    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel> {
        let doc = match (user_a, user_b) {
            self_user if self_user.0 == self_user.1 => {
                doc! {
                    "channel_type": "SavedMessages",
                    "user": self_user.0
                }
            }
            users => {
                doc! {
                    "channel_type": "DirectMessage",
                    "recipients": {
                        "$all": [ users.0, users.1 ]
                    }
                }
            }
        };
        query!(self, find_one, COL, doc)?.ok_or_else(|| create_error!(NotFound))
    }

    // Update channel
    async fn update_channel(
        &self,
        id: &str,
        channel: &PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            id,
            channel,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None
        )
        .map(|_| ())
    }

    // Remove a user from a group
    async fn remove_user_from_group(&self, channel: &str, user: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": channel
                },
                doc! {
                    "$pull": {
                        "recipients": user
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", "channels"))
    }

    async fn delete_channel(&self, channel: &Channel) -> Result<()> {
        query!(self, delete_one_by_id, COL, &channel.id()).map(|_| ())
    }
}

impl IntoDocumentPath for FieldsChannel {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsChannel::Description => "description",
            FieldsChannel::Icon => "icon",
            FieldsChannel::DefaultPermissions => "default_permissions",
        })
    }
}
