use crate::database::*;
use crate::util::result::Result;

use rocket_contrib::json::JsonValue;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum InviteResponse {
    Server {
        server_id: String,
        server_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_icon: Option<File>,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_banner: Option<File>,
        channel_id: String,
        channel_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_description: Option<String>,
        user_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_avatar: Option<File>,
        member_count: i64,
    },
}

#[get("/<target>")]
pub async fn req(target: Ref) -> Result<JsonValue> {
    let target = target.fetch_invite().await?;

    match target {
        Invite::Server {
            channel, creator, ..
        } => {
            let channel = Ref::from_unchecked(channel).fetch_channel().await?;
            let creator = Ref::from_unchecked(creator).fetch_user().await?;

            match channel {
                Channel::TextChannel { id, server, name, description, .. }
                | Channel::VoiceChannel { id, server, name, description, .. } => {
                    let server = Ref::from_unchecked(server).fetch_server().await?;
    
                    Ok(json!(InviteResponse::Server {
                        member_count: Server::get_member_count(&server.id).await?,
                        server_id: server.id,
                        server_name: server.name,
                        server_icon: server.icon,
                        server_banner: server.banner,
                        channel_id: id,
                        channel_name: name,
                        channel_description: description,
                        user_name: creator.username,
                        user_avatar: creator.avatar
                    }))
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!(),
    }
}
