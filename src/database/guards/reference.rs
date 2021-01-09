use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::{doc, from_bson, Bson};
use rocket::http::RawStr;
use rocket::request::FromParam;
use serde::{Deserialize, Serialize};
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

    pub async fn fetch_user(&self) -> Result<User> {
        let doc = get_collection("users")
            .find_one(
                doc! {
                    "_id": &self.id
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: "user",
            })?
            .ok_or_else(|| Error::UnknownUser)?;

        Ok(
            from_bson(Bson::Document(doc)).map_err(|_| Error::DatabaseError {
                operation: "from_bson",
                with: "user",
            })?,
        )
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
