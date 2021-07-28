use std::collections::HashMap;

use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use futures::StreamExt;
use mongodb::bson::{Bson, doc};
use mongodb::bson::from_document;
use mongodb::bson::to_document;
use mongodb::bson::Document;
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>
}

pub type PermissionTuple = (
    i32, // server permission
    i32  // channel permission
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: PermissionTuple,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>
    // Bri'ish API conventions
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub id: String,
    pub title: String,
    pub channels: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ban {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemMessageChannels {
    pub user_joined: Option<String>,
    pub user_left: Option<String>,
    pub user_kicked: Option<String>,
    pub user_banned: Option<String>,
}

pub enum RemoveMember {
    Leave,
    Kick,
    Ban,
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
    pub categories: Option<Vec<Category>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,

    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub roles: HashMap<String, Role>,
    pub default_permissions: PermissionTuple,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<File>,
}

impl Server {
    pub async fn create(self) -> Result<()> {
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
        // Check if there are any attachments we need to delete.
        Channel::delete_messages(Bson::Document(doc! { "$in": &self.channels })).await?;

        // Delete all channels.
        get_collection("channels")
            .delete_many(
                doc! {
                    "server": &self.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_many",
                with: "channels",
            })?;

        // Delete any associated objects, e.g. unreads and invites.
        Channel::delete_associated_objects(Bson::Document(doc! { "$in": &self.channels })).await?;

        // Delete members and bans.
        for with in &["server_members", "server_bans"] {
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

        // Delete server icon / banner.
        if let Some(attachment) = &self.icon {
            attachment.delete().await?;
        }

        if let Some(attachment) = &self.banner {
            attachment.delete().await?;
        }

        // Delete the server
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

        Ok(())
    }

    pub async fn fetch_members(id: &str) -> Result<Vec<Member>> {
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
            .filter_map(|x| from_document(x).ok())
            .collect::<Vec<Member>>())
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

        if let Some(channels) = &self.system_messages {
            if let Some(cid) = &channels.user_joined {
                let channel = Ref::from_unchecked(cid.clone()).fetch_channel().await?;
                Content::SystemMessage(SystemMessage::UserJoined { id: id.to_string() })
                    .send_as_system(&channel)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn remove_member(&self, id: &str, removal: RemoveMember) -> Result<()> {
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

            if let Some(channels) = &self.system_messages {
                let message = match removal {
                    RemoveMember::Leave => {
                        if let Some(cid) = &channels.user_left {
                            Some((cid.clone(), SystemMessage::UserLeft { id: id.to_string() }))
                        } else {
                            None
                        }
                    }
                    RemoveMember::Kick => {
                        if let Some(cid) = &channels.user_kicked {
                            Some((
                                cid.clone(),
                                SystemMessage::UserKicked { id: id.to_string() },
                            ))
                        } else {
                            None
                        }
                    }
                    RemoveMember::Ban => {
                        if let Some(cid) = &channels.user_banned {
                            Some((
                                cid.clone(),
                                SystemMessage::UserBanned { id: id.to_string() },
                            ))
                        } else {
                            None
                        }
                    }
                };

                if let Some((cid, message)) = message {
                    let channel = Ref::from_unchecked(cid).fetch_channel().await?;
                    Content::SystemMessage(message)
                        .send_as_system(&channel)
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn get_member_count(id: &str) -> Result<i64> {
        Ok(get_collection("server_members")
            .count_documents(
                doc! {
                    "_id.server": id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "count_documents",
                with: "server_members",
            })?)
    }
}
