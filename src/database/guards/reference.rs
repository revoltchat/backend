use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::{doc, from_document};
use rocket::request::FromParam;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Ref {
    #[validate(length(min = 1, max = 26))]
    pub id: String,
}

impl Ref {
    pub fn from_unchecked(id: String) -> Ref {
        Ref { id }
    }

    pub fn from(id: String) -> Result<Ref> {
        let r = Ref { id };
        r.validate()
            .map_err(|error| Error::FailedValidation { error })?;
        Ok(r)
    }

    async fn fetch<T: DeserializeOwned>(&self, collection: &'static str) -> Result<T> {
        let doc = get_collection(&collection)
            .find_one(
                doc! {
                    "_id": &self.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: &collection,
            })?
            .ok_or_else(|| Error::NotFound)?;

        Ok(from_document::<T>(doc).map_err(|_| Error::DatabaseError {
            operation: "from_document",
            with: &collection,
        })?)
    }

    pub async fn fetch_user(&self) -> Result<User> {
        self.fetch("users").await
    }

    pub async fn fetch_channel(&self) -> Result<Channel> {
        self.fetch("channels").await
    }

    pub async fn fetch_server(&self) -> Result<Server> {
        self.fetch("servers").await
    }

    pub async fn fetch_invite(&self) -> Result<Invite> {
        self.fetch("channel_invites").await
    }

    pub async fn fetch_bot(&self) -> Result<Bot> {
        self.fetch("bots").await
    }

    pub async fn fetch_member(&self, server: &str) -> Result<Member> {
        let doc = get_collection("server_members")
            .find_one(
                doc! {
                    "_id.user": &self.id,
                    "_id.server": server
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "server_member",
            })?
            .ok_or_else(|| Error::NotFound)?;

        Ok(
            from_document::<Member>(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "server_member",
            })?,
        )
    }

    pub async fn fetch_ban(&self, server: &str) -> Result<Ban> {
        let doc = get_collection("server_bans")
            .find_one(
                doc! {
                    "_id.user": &self.id,
                    "_id.server": server
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "server_ban",
            })?
            .ok_or_else(|| Error::NotFound)?;

        Ok(from_document::<Ban>(doc).map_err(|_| Error::DatabaseError {
            operation: "from_document",
            with: "server_ban",
        })?)
    }

    pub async fn fetch_message(&self, channel: &Channel) -> Result<Message> {
        let message: Message = self.fetch("messages").await?;
        if &message.channel != channel.id() {
            Err(Error::InvalidOperation)
        } else {
            Ok(message)
        }
    }
}

impl User {
    pub fn as_ref(&self) -> Ref {
        Ref {
            id: self.id.to_string(),
        }
    }
}

impl<'r> FromParam<'r> for Ref {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if let Ok(result) = Ref::from(param.to_string()) {
            if result.validate().is_ok() {
                return Ok(result);
            }
        }

        Err(param)
    }
}
