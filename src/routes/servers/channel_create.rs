use std::collections::HashMap;

use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Serialize, Deserialize)]
enum ChannelType {
    Text,
    Voice
}

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::Text
    }
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "type", default = "ChannelType::default")]
    channel_type: ChannelType,
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    // Maximum length of 36 allows both ULIDs and UUIDs.
    #[validate(length(min = 1, max = 36))]
    nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

#[post("/<target>/channels", data = "<info>")]
pub async fn req(user: User, target: Ref, info: Json<Data>) -> Result<Value> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let target = target.fetch_server().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_channels() {
        Err(Error::MissingPermission)?
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
    let channel = match info.channel_type {
        ChannelType::Text => Channel::TextChannel {
            id: id.clone(),
            server: target.id.clone(),
            nonce: Some(info.nonce),

            name: info.name,
            description: info.description,
            icon: None,
            last_message_id: None,

            default_permissions: None,
            role_permissions: HashMap::new(),
            
            nsfw: info.nsfw.unwrap_or_default(),
        },
        ChannelType::Voice => Channel::VoiceChannel {
            id: id.clone(),
            server: target.id.clone(),
            nonce: Some(info.nonce),

            name: info.name,
            description: info.description,
            icon: None,

            default_permissions: None,
            role_permissions: HashMap::new(),

            nsfw: info.nsfw.unwrap_or_default()
        }
    };

    channel.clone().publish().await?;
    get_collection("servers")
        .update_one(
            doc! {
                "_id": target.id
            },
            doc! {
                "$addToSet": {
                    "channels": id
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "server",
        })?;

    Ok(json!(channel))
}
