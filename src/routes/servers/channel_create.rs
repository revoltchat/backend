use std::collections::HashMap;

use revolt_quark::{
    models::{server::PartialServer, Channel, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

/// # Channel Type
#[derive(Serialize, Deserialize, JsonSchema)]
enum ChannelType {
    /// Text Channel
    Text,
    /// Voice Channel
    Voice,
}

impl Default for ChannelType {
    fn default() -> Self {
        ChannelType::Text
    }
}

/// # Channel Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataCreateChannel {
    /// Channel type
    #[serde(rename = "type", default = "ChannelType::default")]
    channel_type: ChannelType,
    /// Channel name
    #[validate(length(min = 1, max = 32))]
    name: String,
    /// Channel description
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    /// Whether this channel is age restricted
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

/// # Create Channel
///
/// Create a new Text or Voice channel.
#[openapi(tag = "Server Information")]
#[post("/<target>/channels", data = "<info>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    info: Json<DataCreateChannel>,
) -> Result<Json<Channel>> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManageChannel)
        .await?;

    let id = Ulid::new().to_string();
    let mut channels = server.channels.clone();
    channels.push(id.clone());

    let DataCreateChannel {
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
