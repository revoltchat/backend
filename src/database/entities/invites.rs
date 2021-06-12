use mongodb::bson::doc;
use mongodb::bson::from_document;
use mongodb::bson::to_document;
use serde::{Deserialize, Serialize};

use crate::database::get_collection;
use crate::util::result::Error;
use crate::util::result::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Invite {
    Server {
        #[serde(rename = "_id")]
        code: String,
        server: String,
        creator: String,
        channel: String,
    },
    Group {
        #[serde(rename = "_id")]
        code: String,
        creator: String,
        channel: String,
    }, /* User {
           code: String,
           user: String
       } */
}

impl Invite {
    pub fn code(&self) -> &String {
        match &self {
            Invite::Server { code, .. } => code,
            Invite::Group { code, .. } => code,
        }
    }

    pub fn creator(&self) -> &String {
        match &self {
            Invite::Server { creator, .. } => creator,
            Invite::Group { creator, .. } => creator,
        }
    }

    pub async fn get(code: &str) -> Result<Invite> {
        let doc = get_collection("channel_invites")
            .find_one(doc! { "_id": code }, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "invite",
            })?
            .ok_or_else(|| Error::UnknownServer)?;

        from_document::<Invite>(doc).map_err(|_| Error::DatabaseError {
            operation: "from_document",
            with: "invite",
        })
    }

    pub async fn save(self) -> Result<()> {
        get_collection("channel_invites")
            .insert_one(
                to_document(&self).map_err(|_| Error::DatabaseError {
                    operation: "to_bson",
                    with: "invite",
                })?,
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "insert_one",
                with: "invite",
            })?;

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        get_collection("channel_invites")
            .delete_one(
                doc! {
                    "_id": self.code()
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "delete_one",
                with: "invite",
            })?;

        Ok(())
    }
}
