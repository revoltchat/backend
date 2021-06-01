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
    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invite {
    pub code: String,
    pub creator: String,
    pub channel: String,
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
    pub channels: Vec<String>,
    // pub invites: Vec<Invite>,
    // pub bans: Vec<Ban>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<File>,
}

impl Server {
    pub async fn get(id: &str) -> Result<Server> {
        let doc = get_collection("servers")
            .find_one(doc! { "_id": id }, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "server",
            })?
            .ok_or_else(|| Error::UnknownServer)?;

        from_document::<Server>(doc).map_err(|_| Error::DatabaseError {
            operation: "from_document",
            with: "server",
        })
    }

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
}
