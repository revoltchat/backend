use std::{collections::HashSet, iter::FromIterator};

use revolt_quark::{
    get_relationship,
    models::{user::RelationshipStatus, Channel, User},
    Db, Error, Result,
};

use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use crate::util::variables::MAX_GROUP_SIZE;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    users: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

#[post("/create", data = "<info>")]
pub async fn req(db: &Db, user: User, info: Json<Data>) -> Result<Json<Channel>> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut set: HashSet<String> = HashSet::from_iter(info.users.into_iter());
    set.insert(user.id.clone());

    if set.len() > *MAX_GROUP_SIZE {
        return Err(Error::GroupTooLarge {
            max: *MAX_GROUP_SIZE,
        });
    }

    for target in &set {
        match get_relationship(&user, target) {
            RelationshipStatus::Friend | RelationshipStatus::User => {}
            _ => {
                return Err(Error::NotFriends);
            }
        }
    }

    let group = Channel::Group {
        id: Ulid::new().to_string(),

        name: info.name,
        owner: user.id,
        description: info.description,
        recipients: set.into_iter().collect::<Vec<String>>(),

        icon: None,
        last_message_id: None,

        permissions: None,

        nsfw: info.nsfw.unwrap_or(false),
    };

    db.insert_channel(&group).await?;
    Ok(Json(group))
}
