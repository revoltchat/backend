use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result, EmptyResponse};
use crate::{database::*, notifications::events::RemoveBotField};
use crate::util::regex::RE_USERNAME;

use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    public: Option<bool>,
    interactions_url: Option<String>,
    remove: Option<RemoveBotField>,
}

#[patch("/<target>", data = "<data>")]
pub async fn edit_bot(user: User, target: Ref, data: Json<Data>) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if data.name.is_none()
        && data.public.is_none()
        && data.interactions_url.is_none()
        && data.remove.is_none()
    {
        return Ok(EmptyResponse {});
    }

    let bot = target.fetch_bot().await?;
    if bot.owner != user.id {
        return Err(Error::MissingPermission);
    }

    if let Some(name) = &data.name {
        if User::is_username_taken(&name).await? {
            return Err(Error::UsernameTaken);
        }
        db_conn().update_username(&target.id, name).await?;
        ClientboundNotification::UserUpdate {
            id: target.id.clone(),
            data: json!({
                "username": name
            }),
            clear: None,
        }
        .publish_as_user(target.id.clone());
    }

    let mut set = doc! {};
    let mut unset = doc! {};

    if let Some(remove) = &data.remove {
        match remove {
            RemoveBotField::InteractionsURL => {
                unset.insert("interactions_url", 1);
            }
        }
    }

    if let Some(public) = &data.public {
        set.insert("public", public);
    }

    if let Some(interactions_url) = &data.interactions_url {
        set.insert("interactions_url", interactions_url);
    }

    let mut operations = doc! {};
    if set.len() > 0 {
        operations.insert("$set", &set);
    }

    if unset.len() > 0 {
        operations.insert("$unset", unset);
    }

    if operations.len() > 0 {
        db_conn().apply_bot_changes(&target.id, operations).await?;
    }

    Ok(EmptyResponse {})
}
