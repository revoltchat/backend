use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    // Maximum length of 36 allows both ULIDs and UUIDs.
    #[validate(length(min = 1, max = 36))]
    nonce: String,
}

#[post("/create", data = "<info>")]
pub async fn req(user: User, info: Json<Data>) -> Result<JsonValue> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if get_collection("servers")
        .find_one(
            doc! {
                "nonce": &info.nonce
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "server",
        })?
        .is_some()
    {
        Err(Error::DuplicateNonce)?
    }

    let id = Ulid::new().to_string();
    let cid = Ulid::new().to_string();

    let server = Server {
        id: id.clone(),
        nonce: Some(info.nonce.clone()),
        owner: user.id.clone(),

        name: info.name,
        description: info.description,
        channels: vec![cid.clone()],

        icon: None,
        banner: None,
    };

    Channel::TextChannel {
        id: cid,
        server: id,
        nonce: Some(info.nonce),
        name: "general".to_string(),
        description: None,
        icon: None,
        last_message: None
    }
    .publish()
    .await?;

    server.clone().create().await?;
    server.join_member(&user.id).await?;

    Ok(json!(server))
}
