use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::{doc, from_bson, from_document, Bson};
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Ref {
    #[validate(length(min = 26, max = 26))]
    pub id: String,
}

impl Ref {
    pub fn from(id: String) -> Result<Ref> {
        Ok(Ref { id })
    }

    pub async fn fetch<T: DeserializeOwned>(&self, collection: &'static str) -> Result<T> {
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
            .ok_or_else(|| Error::UnknownUser)?;

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

    pub async fn fetch_message(&self) -> Result<Message> {
        self.fetch("messages").await
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
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        if let Ok(result) = Ref::from(param.to_string()) {
            if result.validate().is_ok() {
                return Ok(result);
            }
        }

        Err(param)
    }
}
