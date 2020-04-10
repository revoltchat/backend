use super::channel::ChannelType;
use super::Response;
use crate::database::{self, channel::Channel, PermissionCalculator, Permission};
use crate::guards::auth::UserRef;
use crate::guards::guild::GuildRef;

use bson::{doc, from_bson, Bson};
use mongodb::options::FindOptions;
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
    if let Ok(result) = database::get_collection("members").find(
        doc! {
            "_id.user": &user.id
        },
        None,
    ) {
        let mut guilds = vec![];
        for item in result {
            if let Ok(entry) = item {
                guilds.push(Bson::String(
                    entry
                        .get_document("_id")
                        .unwrap()
                        .get_str("guild")
                        .unwrap()
                        .to_string(),
                ));
            }
        }

        if let Ok(result) = database::get_collection("guilds").find(
            doc! {
                "_id": {
                    "$in": guilds
                }
            },
            FindOptions::builder()
                .projection(doc! {
                    "_id": 1,
                    "name": 1,
                    "description": 1,
                    "owner": 1,
                })
                .build(),
        ) {
            let mut parsed = vec![];
            for item in result {
                let doc = item.unwrap();
                parsed.push(json!({
                    "id": doc.get_str("_id").unwrap(),
                    "name": doc.get_str("name").unwrap(),
                    "description": doc.get_str("description").unwrap(),
                    "owner": doc.get_str("owner").unwrap(),
                }));
            }

            Response::Success(json!(parsed))
        } else {
            Response::InternalServerError(json!({ "error": "Failed to fetch guilds." }))
        }
    } else {
        Response::InternalServerError(json!({ "error": "Failed to fetch memberships." }))
    }
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
                if let Ok(entry) = item {
                    if let Ok(channel) =
                        from_bson(bson::Bson::Document(entry)) as Result<Channel, _>
                    {
                        channels.push(json!({
                            "id": channel.id,
                            "last_message": channel.last_message,
                            "name": channel.name,
                            "description": channel.description,
                        }));
                    }
                }
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
    if col
        .find_one(doc! { "nonce": nonce.clone() }, None)
        .unwrap()
        .is_some()
    {
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

    if database::get_collection("members")
        .insert_one(
            doc! {
                "_id": {
                    "guild": &id,
                    "user": &user.id
                }
            },
            None,
        )
        .is_err()
    {
        return Response::InternalServerError(
            json!({ "error": "Failed to add you to members list." }),
        );
    }

    if col
        .insert_one(
            doc! {
                "_id": &id,
                "nonce": nonce,
                "name": name,
                "description": description,
                "owner": &user.id,
                "channels": [
                    &channel_id
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

/// fetch a guild's member
#[get("/<target>/members")]
pub fn fetch_members(user: UserRef, target: GuildRef) -> Option<Response> {
    with_permissions!(user, target);

    if let Ok(result) =
        database::get_collection("members").find(doc! { "_id.guild": target.id }, None)
    {
        let mut users = vec![];

        for item in result {
            if let Ok(doc) = item {
                users.push(json!({
                    "id": doc.get_document("_id").unwrap().get_str("user").unwrap(),
                    "nickname": doc.get_str("nickname").ok(),
                }));
            }
        }

        Some(Response::Success(json!(users)))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to fetch members." }),
        ))
    }
}

/// fetch a guild member
#[get("/<target>/members/<other>")]
pub fn fetch_member(user: UserRef, target: GuildRef, other: String) -> Option<Response> {
    with_permissions!(user, target);

    if let Ok(result) = database::get_collection("members").find_one(
        doc! {
            "_id.guild": &target.id,
            "_id.user": &other,
        },
        None,
    ) {
        if let Some(doc) = result {
            Some(Response::Success(json!({
                "id": doc.get_document("_id").unwrap().get_str("user").unwrap(),
                "nickname": doc.get_str("nickname").ok(),
            })))
        } else {
            Some(Response::NotFound(
                json!({ "error": "User not part of guild." }),
            ))
        }
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to fetch member." }),
        ))
    }
}

/// kick a guild member
#[delete("/<target>/members/<other>")]
pub fn kick_member(user: UserRef, target: GuildRef, other: String) -> Option<Response> {
    let permissions = with_permissions!(user, target);

    if !permissions.get_kick_members() {
        return Some(Response::LackingPermission(Permission::KickMembers));
    }

    if user.id == other {
        return Some(Response::BadRequest(json!({ "error": "Cannot kick yourself." })))
    }

    if database::get_collection("members").delete_one(
        doc! {
            "_id.guild": &target.id,
            "_id.user": &other,
        },
        None,
    ).is_ok() {
        Some(Response::Result(super::Status::Ok))
    } else {
        Some(Response::InternalServerError(
            json!({ "error": "Failed to kick member." }),
        ))
    }
}
