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
    // Maximum length of 36 allows both ULIDs and UUIDs.
    #[validate(length(min = 1, max = 36))]
    nonce: String,
}

#[post("/create", data = "<info>")]
pub async fn req(user: User, info: Json<Data>) -> Result<JsonValue> {
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
    
    Channel::TextChannel {
        id: cid.clone(),
        server: id.clone(),
        nonce: Some(info.nonce.clone()),
        name: "general".to_string(),
        description: None,
        icon: None,
    }.publish().await?;

    let server = Server {
        id: id.clone(),
        nonce: Some(info.nonce.clone()),
        owner: user.id.clone(),

        name: info.name.clone(),
        channels: vec![ cid ],

        icon: None,
        banner: None,
    };

    get_collection("server_members")
        .insert_one(
            doc! {
                "_id": {
                    "server": id,
                    "user": user.id
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "insert_one",
            with: "server_members",
        })?;

    server.clone().publish().await?;

    Ok(json!(server))
}
