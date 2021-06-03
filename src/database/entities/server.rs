use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use mongodb::bson::doc;
use mongodb::bson::from_document;
use mongodb::bson::to_document;
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
    pub id: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub owner: String,

    pub name: String,
    // pub default_permissions: u32,
    // pub invites: Vec<Invite>,
    // pub bans: Vec<Ban>,
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
        unimplemented!()
    }

    pub async fn join_member(&self, id: &str) -> Result<()> {
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
            user: id.to_string()
        }
        .publish(self.id.clone());

        Ok(())
    }
}
