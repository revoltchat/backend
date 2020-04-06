use crate::database::{self, user::User};

use bson::{bson, doc};
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::channel::ChannelType;

#[derive(Serialize, Deserialize)]
pub struct CreateGuild {
    name: String,
    description: Option<String>,
    nonce: String,
}

/// send a message to a channel
#[post("/create", data = "<info>")]
pub fn create_guild(user: User, info: Json<CreateGuild>) -> JsonValue {
    if !user.email_verification.verified {
        return json!({
            "success": false,
            "error": "Email not verified!",
        });
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
        return json!({
            "success": false,
            "error": "Guild already created!"
        });
    }

    let channel_id = Ulid::new().to_string();
    if let Err(_) = channels.insert_one(
        doc! {
            "_id": channel_id.clone(),
            "channel_type": ChannelType::GUILDCHANNEL as u32,
            "name": "general",
        },
        None,
    ) {
        return json!({
            "success": false,
            "error": "Failed to create guild channel."
        });
    }

    let id = Ulid::new().to_string();
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
                    user.id
                ],
                "invites": [],
            },
            None,
        )
        .is_ok()
    {
        json!({
            "success": true,
            "id": id,
        })
    } else {
        channels
            .delete_one(doc! { "_id": channel_id }, None)
            .expect("Failed to delete the channel we just made.");

        json!({
            "success": false,
            "error": "Failed to create guild."
        })
    }
}
