use std::collections::HashSet;

use bson::{Bson, Document};

use crate::models::channel::{Channel, FieldsChannel, PartialChannel};
use crate::r#impl::mongo::IntoDocumentPath;
use crate::{AbstractChannel, AbstractServer, Error, OverrideField, Result};

use super::super::MongoDb;

static COL: &str = "channels";

impl MongoDb {
    pub async fn delete_associated_channel_objects(&self, id: Bson) -> Result<()> {
        // Delete all invites to these channels.
        self.col::<Document>("channel_invites")
            .delete_many(
                doc! {
                    "channel": &id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "channel_invites",
            })?;

        // Delete unread message objects on channels.
        self.col::<Document>("channel_unreads")
            .delete_many(
                doc! {
                    "_id.channel": &id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "channel_unreads",
            })
            .map(|_| ())

        // update many attachments with parent id
    }
}

#[async_trait]
impl AbstractChannel for MongoDb {
    async fn fetch_channel(&self, id: &str) -> Result<Channel> {
        self.find_one_by_id(COL, id).await
    }

    async fn fetch_channels<'a>(&self, ids: &'a [String]) -> Result<Vec<Channel>> {
        self.find(
            COL,
            doc! {
                "_id": {
                    "$in": ids
                }
            },
        )
        .await
    }

    async fn insert_channel(&self, channel: &Channel) -> Result<()> {
        self.insert_one(COL, channel).await.map(|_| ())
    }

    async fn update_channel(
        &self,
        id: &str,
        channel: &PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        self.update_one_by_id(
            COL,
            id,
            channel,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
    }

    async fn delete_channel(&self, channel: &Channel) -> Result<()> {
        let id = channel.id().to_string();
        let server_id = match channel {
            Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => {
                Some(server)
            }
            _ => None,
        };

        // Delete invites and unreads.
        self.delete_associated_channel_objects(Bson::String(id.to_string()))
            .await?;

        // Delete messages.
        self.delete_bulk_messages(doc! {
            "channel": &id
        })
        .await?;

        // Remove from server object.
        if let Some(server) = server_id {
            let server = self.fetch_server(server).await?;
            let mut update = doc! {
                "$pull": {
                    "channels": &id
                }
            };

            if let Some(sys) = &server.system_messages {
                let mut unset = doc! {};

                if let Some(cid) = &sys.user_joined {
                    if &id == cid {
                        unset.insert("system_messages.user_joined", 1_i32);
                    }
                }

                if let Some(cid) = &sys.user_left {
                    if &id == cid {
                        unset.insert("system_messages.user_left", 1_i32);
                    }
                }

                if let Some(cid) = &sys.user_kicked {
                    if &id == cid {
                        unset.insert("system_messages.user_kicked", 1_i32);
                    }
                }

                if let Some(cid) = &sys.user_banned {
                    if &id == cid {
                        unset.insert("system_messages.user_banned", 1_i32);
                    }
                }

                if !unset.is_empty() {
                    update.insert("$unset", unset);
                }
            }

            self.col::<Document>("servers")
                .update_one(
                    doc! {
                        "_id": server.id
                    },
                    update,
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "update_one",
                    with: "servers",
                })?;
        }

        // Delete associated attachments
        self.delete_many_attachments(doc! {
            "object_id": &id
        })
        .await?;

        // Delete the channel itself
        self.delete_one_by_id(COL, &id).await.map(|_| ())
    }

    async fn find_direct_messages(&self, user_id: &str) -> Result<Vec<Channel>> {
        self.find(
            COL,
            doc! {
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
            },
        )
        .await
    }

    async fn find_saved_messages_channel(&self, user_id: &str) -> Result<Channel> {
        self.find_one(
            COL,
            doc! {
                "channel_type": "SavedMessages",
                "user": user_id
            },
        )
        .await
    }

    async fn find_direct_message_channel(&self, user_a: &str, user_b: &str) -> Result<Channel> {
        self.find_one(
            COL,
            if user_a == user_b {
                doc! {
                    "channel_type": "SavedMessages",
                    "user": user_a
                }
            } else {
                doc! {
                    "channel_type": "DirectMessage",
                    "recipients": {
                        "$all": [ user_a, user_b ]
                    }
                }
            },
        )
        .await
    }

    async fn add_user_to_group(&self, channel: &str, user: &str) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": channel
                },
                doc! {
                    "$push": {
                        "recipients": user
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })
    }

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
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })
    }

    async fn set_channel_role_permission(
        &self,
        channel: &str,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! { "_id": channel },
                doc! {
                    "$set": {
                        "role_permissions.".to_owned() + role: permissions
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "channel",
            })
    }

    async fn check_channels_exist(&self, channels: &HashSet<String>) -> Result<bool> {
        let count = channels.len() as u64;
        self.col::<Document>(COL)
            .count_documents(
                doc! {
                    "_id": {
                        "$in": channels.iter().cloned().collect::<Vec<String>>()
                    }
                },
                None,
            )
            .await
            .map(|x| x == count)
            .map_err(|_| Error::DatabaseError {
                operation: "count_documents",
                with: "channel",
            })
    }
}

impl IntoDocumentPath for FieldsChannel {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsChannel::DefaultPermissions => "default_permissions",
            FieldsChannel::Description => "description",
            FieldsChannel::Icon => "icon",
        })
    }
}
