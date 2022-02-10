use revolt_quark::models::{Channel, File, Invite};
use revolt_quark::{Db, Ref, Result};

use rocket::serde::json::Json;
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
pub async fn req(db: &Db, target: Ref) -> Result<Json<InviteResponse>> {
    Ok(Json(match target.as_invite(db).await? {
        Invite::Server {
            channel, creator, ..
        } => {
            let channel = db.fetch_channel(&channel).await?;
            let user = db.fetch_user(&creator).await?;

            match channel {
                Channel::TextChannel {
                    id,
                    server,
                    name,
                    description,
                    ..
                }
                | Channel::VoiceChannel {
                    id,
                    server,
                    name,
                    description,
                    ..
                } => {
                    let server = db.fetch_server(&server).await?;

                    InviteResponse::Server {
                        member_count: db.fetch_member_count(&server.id).await? as i64,
                        server_id: server.id,
                        server_name: server.name,
                        server_icon: server.icon,
                        server_banner: server.banner,
                        channel_id: id,
                        channel_name: name,
                        channel_description: description,
                        user_name: user.username,
                        user_avatar: user.avatar,
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => unimplemented!(),
    }))
}
