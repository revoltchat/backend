use revolt_quark::{
    models::{Server, User},
    Db, Error, Result, DEFAULT_PERMISSION_CHANNEL_SERVER, DEFAULT_SERVER_PERMISSION,
};

use mongodb::bson::doc;
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
    
    let server = Server {
        id: Ulid::new().to_string(),
        name,
        description,
        nsfw: nsfw.unwrap_or(false),
        default_permissions: (
            *DEFAULT_SERVER_PERMISSION as i32,
            *DEFAULT_PERMISSION_CHANNEL_SERVER as i32,
        ),
        ..Default::default()
    };

    db.insert_server(&server).await?;
    Ok(Json(server))
}
