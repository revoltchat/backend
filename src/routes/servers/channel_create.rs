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

#[post("/<target>/channels", data = "<info>")]
pub async fn req(user: User, target: Ref, info: Json<Data>) -> Result<JsonValue> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let target = target.fetch_server().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_server() {
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
    let channel = Channel::TextChannel {
        id: id.clone(),
        server: target.id.clone(),
        nonce: Some(info.nonce),

        name: info.name,
        description: info.description,
        icon: None,
        last_message: None
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
