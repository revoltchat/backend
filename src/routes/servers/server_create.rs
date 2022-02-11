use std::collections::HashMap;

use revolt_quark::{
    models::{server_member::MemberCompositeKey, Channel, Member, Server, User},
    Db, Error, Result, DEFAULT_PERMISSION_CHANNEL_SERVER, DEFAULT_SERVER_PERMISSION,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use crate::util::variables::MAX_SERVER_COUNT;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

#[post("/create", data = "<info>")]
pub async fn req(db: &Db, user: User, info: Json<Data>) -> Result<Json<Server>> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if !user.can_acquire_server(db).await? {
        return Err(Error::TooManyServers {
            max: *MAX_SERVER_COUNT,
        });
    }

    let Data {
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

    let server = Server {
        id: server_id,
        owner: user.id,
        name,
        description,
        channels: vec![channel_id],
        nsfw: nsfw.unwrap_or(false),
        default_permissions: (
            *DEFAULT_SERVER_PERMISSION as i32,
            *DEFAULT_PERMISSION_CHANNEL_SERVER as i32,
        ),
        ..Default::default()
    };

    let member = Member {
        id: MemberCompositeKey {
            server: server.id.clone(),
            user: server.owner.clone(),
        },
        ..Default::default()
    };

    db.insert_channel(&channel).await?;
    db.insert_server(&server).await?;
    db.insert_member(&member).await?;
    Ok(Json(server))
}
