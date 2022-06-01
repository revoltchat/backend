use std::{collections::HashSet, iter::FromIterator};

use revolt_quark::{
    get_relationship,
    models::{user::RelationshipStatus, Channel, User},
    variables::delta::MAX_GROUP_SIZE,
    Db, Error, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

/// # Group Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataCreateGroup {
    /// Group name
    #[validate(length(min = 1, max = 32))]
    name: String,
    /// Group description
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    /// Array of user IDs to add to the group
    ///
    /// Must be friends with these users.
    #[validate(length(min = 0, max = 49))]
    users: Vec<String>,
    /// Whether this group is age-restricted
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

/// # Create Group
///
/// Create a new group channel.
#[openapi(tag = "Groups")]
#[post("/create", data = "<info>")]
pub async fn req(db: &Db, user: User, info: Json<DataCreateGroup>) -> Result<Json<Channel>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

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

    group.create(db).await?;
    Ok(Json(group))
}
