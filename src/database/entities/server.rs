use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::to_document;
use mongodb::bson::Document;
use mongodb::options::FindOptions;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemberCompositeKey {
    pub server: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ban {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub owner: String,

    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub channels: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<File>,
}

impl Server {
    pub async fn publish(self) -> Result<()> {
        get_collection("servers")
            .insert_one(
                to_document(&self).map_err(|_| Error::DatabaseError {
                    operation: "to_bson",
                    with: "channel",
                })?,
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "server",
            })?;

        let server_id = self.id.clone();
        ClientboundNotification::ServerCreate(self).publish(server_id);

        Ok(())
    }

    pub async fn publish_update(&self, data: JsonValue) -> Result<()> {
        ClientboundNotification::ServerUpdate {
            id: self.id.clone(),
            data,
            clear: None,
        }
        .publish(self.id.clone());

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        let messages = get_collection("messages");

        // Check if there are any attachments we need to delete.
        // ! FIXME: make this generic and merge with channel delete
        // ! e.g. delete_channel(filter: doc!)
        let message_ids = messages
            .find(
                doc! {
                    "server": &self.id,
                    "attachment": {
                        "$exists": 1
                    }
                },
                FindOptions::builder().projection(doc! { "_id": 1 }).build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "fetch_many",
                with: "messages",
            })?
            .filter_map(async move |s| s.ok())
            .collect::<Vec<Document>>()
            .await
            .into_iter()
            .filter_map(|x| x.get_str("_id").ok().map(|x| x.to_string()))
            .collect::<Vec<String>>();

        // If we found any, mark them as deleted.
        if message_ids.len() > 0 {
            get_collection("attachments")
                .update_many(
                    doc! {
                        "message_id": {
                            "$in": message_ids
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
        messages
            .delete_many(
                doc! {
                    "server": &self.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "messages",
            })?;

        // Delete all channels, members, bans and invites.
        for with in ["channels", "invites"] {
            get_collection(with)
                .delete_many(
                    doc! {
                        "server": &self.id
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "delete_many",
                    with,
                })?;
        }

        for with in ["server_members", "server_bans"] {
            get_collection(with)
                .delete_many(
                    doc! {
                        "_id.server": &self.id
                    },
                    None,
                )
                .await
                .map_err(|_| Error::DatabaseError {
                    operation: "delete_many",
                    with,
                })?;
        }

        get_collection("servers")
            .delete_one(
                doc! {
                    "_id": &self.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "server",
            })?;

        ClientboundNotification::ServerDelete {
            id: self.id.clone(),
        }
        .publish(self.id.clone());

        if let Some(attachment) = &self.icon {
            attachment.delete().await?;
        }

        if let Some(attachment) = &self.banner {
            attachment.delete().await?;
        }

        Ok(())
    }

    pub async fn fetch_member_ids(id: &str) -> Result<Vec<String>> {
        Ok(get_collection("server_members")
            .find(
                doc! {
                    "_id.server": id
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
                    .map(|i| i.get_str("user").ok().map(|x| x.to_string()))
            })
            .flatten()
            .collect::<Vec<String>>())
    }

    pub async fn join_member(&self, id: &str) -> Result<()> {
        if get_collection("server_bans")
            .find_one(
                doc! {
                    "_id.server": &self.id,
                    "_id.user": &id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "server_bans",
            })?
            .is_some()
        {
            return Err(Error::Banned);
        }

        get_collection("server_members")
            .insert_one(
                doc! {
                    "_id": {
                        "server": &self.id,
                        "user": &id
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "server_members",
            })?;

        ClientboundNotification::ServerMemberJoin {
            id: self.id.clone(),
            user: id.to_string(),
        }
        .publish(self.id.clone());

        Ok(())
    }

    pub async fn remove_member(&self, id: &str) -> Result<()> {
        let result = get_collection("server_members")
            .delete_one(
                doc! {
                    "_id": {
                        "server": &self.id,
                        "user": &id
                    }
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "server_members",
            })?;

        if result.deleted_count > 0 {
            ClientboundNotification::ServerMemberLeave {
                id: self.id.clone(),
                user: id.to_string(),
            }
            .publish(self.id.clone());
        }

        Ok(())
    }
}
