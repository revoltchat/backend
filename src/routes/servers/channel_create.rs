use std::collections::HashMap;

use revolt_quark::{
    models::{server::PartialServer, Channel, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Serialize, Deserialize)]
enum ChannelType {
    Text,
    Voice,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

#[post("/<target>/channels", data = "<info>")]
pub async fn req(db: &Db, user: User, target: Ref, info: Json<Data>) -> Result<Json<Channel>> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc(db)
        .await
        .can_manage_channel()
    {
        return Error::from_permission(Permission::ManageChannel);
    }

    let id = Ulid::new().to_string();
    let mut channels = server.channels.clone();
    channels.push(id.clone());

    let Data {
        name,
        description,
        nsfw,
        channel_type,
    } = info;
    let channel = match channel_type {
        ChannelType::Text => Channel::TextChannel {
            id,
            server: server.id.clone(),

            name,
            description,

            icon: None,
            last_message_id: None,

            default_permissions: None,
            role_permissions: HashMap::new(),

            nsfw: nsfw.unwrap_or(false),
        },
        ChannelType::Voice => Channel::VoiceChannel {
            id,
            server: server.id.clone(),

            name,
            description,
            icon: None,

            default_permissions: None,
            role_permissions: HashMap::new(),

            nsfw: nsfw.unwrap_or(false),
        },
    };

    channel.create(db).await?;
    server
        .update(
            db,
            PartialServer {
                channels: Some(channels),
                ..Default::default()
            },
            vec![],
        )
        .await?;

    Ok(Json(channel))
}
