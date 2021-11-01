use crate::database::*;
use crate::util::idempotency::IdempotencyKey;
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
    users: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>
}

#[post("/create", data = "<info>")]
pub async fn req(_idempotency: IdempotencyKey, user: User, info: Json<Data>) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
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

    let id = Ulid::new().to_string();
    let channel = Channel::Group {
        id,
        name: info.name,
        description: info.description,
        owner: user.id,
        recipients: set.into_iter().collect::<Vec<String>>(),
        icon: None,
        last_message_id: None,
        permissions: None,
        nsfw: info.nsfw.unwrap_or_default()
    };

    channel.clone().publish().await?;

    Ok(json!(channel))
}
