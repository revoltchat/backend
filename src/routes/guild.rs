use super::channel::ChannelType;
use super::Response;
use crate::database::{self, channel::Channel, PermissionCalculator};
use crate::guards::auth::UserRef;
use crate::guards::guild::GuildRef;

use bson::{doc, from_bson, Bson};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

macro_rules! with_permissions {
    ($user: expr, $target: expr) => {{
        let permissions = PermissionCalculator::new($user.clone())
            .guild($target.clone())
            .as_permission();

        if !permissions.get_access() {
            return None;
        }

        permissions
    }};
}

/// fetch your guilds
#[get("/@me")]
pub fn my_guilds(user: UserRef) -> Response {
    let col = database::get_collection("guilds");
    let guilds = col
        .find(
            doc! {
                "members": {
                    "$elemMatch": {
                        "id": user.id,
                    }
                }
            },
            None,
        )
        .unwrap();

    let mut parsed = vec![];
    for item in guilds {
        let doc = item.unwrap();
        parsed.push(json!({
            "id": doc.get_str("_id").unwrap(),
            "name": doc.get_str("name").unwrap(),
            "description": doc.get_str("description").unwrap(),
            "owner": doc.get_str("owner").unwrap(),
        }));
    }

    Response::Success(json!(parsed))
}

/// fetch a guild
#[get("/<target>")]
pub fn guild(user: UserRef, target: GuildRef) -> Option<Response> {
    with_permissions!(user, target);

    let mut targets = vec![];
    for channel in target.channels {
        targets.push(Bson::String(channel));
    }

    let col = database::get_collection("channels");
    match col.find(
        doc! {
            "_id": {
                "$in": targets,
            }
        },
        None,
    ) {
        Ok(results) => {
            let mut channels = vec![];
            for item in results {
                let channel: Channel = from_bson(bson::Bson::Document(item.unwrap()))
                    .expect("Failed to unwrap channel.");

                channels.push(json!({
                    "id": channel.id,
                    "last_message": channel.last_message,
                    "name": channel.name,
                    "description": channel.description,
                }));
            }

            Some(Response::Success(json!({
                "id": target.id,
                "name": target.name,
                "description": target.description,
                "owner": target.owner,
                "channels": channels,
            })))
        }
        Err(_) => Some(Response::InternalServerError(
            json!({ "error": "Failed to fetch channels." }),
        )),
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateGuild {
    name: String,
    description: Option<String>,
    nonce: String,
}

/// create a new guild
#[post("/create", data = "<info>")]
pub fn create_guild(user: UserRef, info: Json<CreateGuild>) -> Response {
    if !user.email_verified {
        return Response::Unauthorized(json!({ "error": "Email not verified!" }));
    }

    let name: String = info.name.chars().take(32).collect();
    let description: String = info
        .description
        .clone()
        .unwrap_or("No description.".to_string())
        .chars()
        .take(255)
        .collect();
    let nonce: String = info.nonce.chars().take(32).collect();

    let channels = database::get_collection("channels");
    let col = database::get_collection("guilds");
    if let Some(_) = col.find_one(doc! { "nonce": nonce.clone() }, None).unwrap() {
        return Response::BadRequest(json!({ "error": "Guild already created!" }));
    }

    let id = Ulid::new().to_string();
    let channel_id = Ulid::new().to_string();
    if channels
        .insert_one(
            doc! {
                "_id": channel_id.clone(),
                "type": ChannelType::GUILDCHANNEL as u32,
                "name": "general",
                "guild": id.clone(),
            },
            None,
        )
        .is_err()
    {
        return Response::InternalServerError(
            json!({ "error": "Failed to create guild channel." }),
        );
    }

    if col
        .insert_one(
            doc! {
                "_id": id.clone(),
                "nonce": nonce,
                "name": name,
                "description": description,
                "owner": user.id.clone(),
                "channels": [
                    channel_id.clone()
                ],
                "members": [
                    {
                        "id": user.id,
                    }
                ],
                "invites": [],
                "default_permissions": 51,
            },
            None,
        )
        .is_ok()
    {
        Response::Success(json!({ "id": id }))
    } else {
        channels
            .delete_one(doc! { "_id": channel_id }, None)
            .expect("Failed to delete the channel we just made.");

        Response::InternalServerError(json!({ "error": "Failed to create guild." }))
    }
}
