use std::collections::HashMap;

use crate::database::*;
use crate::util::idempotency::IdempotencyKey;
use crate::util::result::{Error, Result};
use crate::util::variables::MAX_SERVER_COUNT;

use mongodb::bson::doc;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>
}

#[post("/create", data = "<info>")]
pub async fn req(_idempotency: IdempotencyKey, user: User, info: Json<Data>) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }

    if !User::can_acquire_server(&user.id).await? {
        Err(Error::TooManyServers {
            max: *MAX_SERVER_COUNT,
        })?
    }
    
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let id = Ulid::new().to_string();
    let cid = Ulid::new().to_string();

    let server = Server {
        id: id.clone(),
        owner: user.id.clone(),

        name: info.name,
        description: info.description,

        channels: vec![cid.clone()],
        categories: None,
        system_messages: Some(SystemMessageChannels {
            user_joined: Some(cid.clone()),
            user_left: Some(cid.clone()),
            user_kicked: Some(cid.clone()),
            user_banned: Some(cid.clone()),
        }),

        roles: HashMap::new(),
        default_permissions: (
            *permissions::server::DEFAULT_PERMISSION as i32,
            *permissions::channel::DEFAULT_PERMISSION_SERVER as i32
        ),

        icon: None,
        banner: None,

        flags: None,
        nsfw: info.nsfw.unwrap_or_default(),
        analytics: false,
        discoverable: false,
    };

    Channel::TextChannel {
        id: cid,
        server: id,
        name: "general".to_string(),
        description: None,

        icon: None,
        last_message_id: None,

        default_permissions: None,
        role_permissions: HashMap::new(),
        nsfw: false
    }
    .publish()
    .await?;

    server.clone().create().await?;
    server.join_member(&user.id).await?;

    Ok(json!(server))
}
