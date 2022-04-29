use std::collections::HashMap;

use revolt_quark::{
    models::{Channel, Server, User},
    variables::delta::MAX_SERVER_COUNT,
    Db, Error, Result, DEFAULT_PERMISSION_SERVER,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

/// # Server Data
#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataCreateServer {
    /// Server name
    #[validate(length(min = 1, max = 32))]
    name: String,
    /// Server description
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    /// Whether this server is age-restricted
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

/// # Create Server Response
#[derive(Validate, Serialize, JsonSchema)]
pub struct CreateServerResponse {
    /// Server object
    server: Server,
    /// Default channels
    channels: Vec<Channel>,
}

/// # Create Server
///
/// Create a new server.
#[openapi(tag = "Server Information")]
#[post("/create", data = "<info>")]
pub async fn req(
    db: &Db,
    user: User,
    info: Json<DataCreateServer>,
) -> Result<Json<CreateServerResponse>> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if !user.can_acquire_server(db).await? {
        return Err(Error::TooManyServers {
            max: *MAX_SERVER_COUNT,
        });
    }

    let DataCreateServer {
        name,
        description,
        nsfw,
    } = info;

    let channel_id = Ulid::new().to_string();
    let server_id = Ulid::new().to_string();

    let channel = Channel::TextChannel {
        id: channel_id.clone(),
        server: server_id.clone(),

        name: "General".into(),
        description: None,

        icon: None,
        last_message_id: None,

        default_permissions: None,
        role_permissions: HashMap::new(),

        nsfw: nsfw.unwrap_or(false),
    };

    db.insert_channel(&channel).await?;

    let server = Server {
        id: server_id.clone(),
        owner: user.id.clone(),
        name,
        description,
        channels: vec![channel_id],
        nsfw: nsfw.unwrap_or(false),
        default_permissions: *DEFAULT_PERMISSION_SERVER as i64,
        ..Default::default()
    };

    server.create(db).await?;
    let channels = server.create_member(db, user, Some(vec![channel])).await?;
    Ok(Json(CreateServerResponse { server, channels }))
}
