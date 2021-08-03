use crate::database::*;
use crate::util::result::{Error, Result};
use crate::util::variables::MAX_GROUP_SIZE;

use mongodb::bson::doc;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::FromIterator;
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    // Maximum length of 36 allows both ULIDs and UUIDs.
    #[validate(length(min = 1, max = 36))]
    nonce: String,
    users: Vec<String>,
}

#[post("/create", data = "<info>")]
pub async fn req(user: User, info: Json<Data>) -> Result<Value> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut set: HashSet<String> = HashSet::from_iter(info.users.iter().cloned());
    set.insert(user.id.clone());

    if set.len() > *MAX_GROUP_SIZE {
        Err(Error::GroupTooLarge {
            max: *MAX_GROUP_SIZE,
        })?
    }

    for target in &set {
        match get_relationship(&user, target) {
            RelationshipStatus::Friend | RelationshipStatus::User => {}
            _ => {
                return Err(Error::NotFriends);
            }
        }
    }

    if get_collection("channels")
        .find_one(
            doc! {
                "nonce": &info.nonce
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "channel",
        })?
        .is_some()
    {
        Err(Error::DuplicateNonce)?
    }

    let id = Ulid::new().to_string();
    let channel = Channel::Group {
        id,
        nonce: Some(info.nonce),
        name: info.name,
        description: info.description,
        owner: user.id,
        recipients: set.into_iter().collect::<Vec<String>>(),
        icon: None,
        last_message: None,
        permissions: None
    };

    channel.clone().publish().await?;

    Ok(json!(channel))
}
