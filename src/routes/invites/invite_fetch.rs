use revolt_quark::models::{Channel, File, Invite};
use revolt_quark::{Db, Ref, Result};

use rocket::serde::json::Json;
use serde::Serialize;

/// # Invite
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum InviteResponse {
    /// Server channel invite
    Server {
        /// Invite code
        code: String,
        /// Id of the server
        server_id: String,
        /// Name of the server
        server_name: String,
        /// Attachment for server icon
        #[serde(skip_serializing_if = "Option::is_none")]
        server_icon: Option<File>,
        /// Attachment for server banner
        #[serde(skip_serializing_if = "Option::is_none")]
        server_banner: Option<File>,
        /// Id of server channel
        channel_id: String,
        /// Name of server channel
        channel_name: String,
        /// Description of server channel
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_description: Option<String>,
        /// Name of user who created the invite
        user_name: String,
        /// Avatar of the user who created the invite
        #[serde(skip_serializing_if = "Option::is_none")]
        user_avatar: Option<File>,
        /// Number of members in this server
        member_count: i64,
    },
    /// Group channel invite
    Group {
        /// Invite code
        code: String,
        /// Id of group channel
        channel_id: String,
        /// Name of group channel
        channel_name: String,
        /// Description of group channel
        #[serde(skip_serializing_if = "Option::is_none")]
        channel_description: Option<String>,
        /// Name of user who created the invite
        user_name: String,
        /// Avatar of the user who created the invite
        #[serde(skip_serializing_if = "Option::is_none")]
        user_avatar: Option<File>,
    },
}

/// # Fetch Invite
///
/// Fetch an invite by its id.
#[openapi(tag = "Invites")]
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
                        code: target.id,
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
        Invite::Group {
            channel, creator, ..
        } => {
            let channel = db.fetch_channel(&channel).await?;
            let user = db.fetch_user(&creator).await?;

            match channel {
                Channel::Group {
                    id,
                    name,
                    description,
                    ..
                } => InviteResponse::Group {
                    code: target.id,
                    channel_id: id,
                    channel_name: name,
                    channel_description: description,
                    user_name: user.username,
                    user_avatar: user.avatar,
                },
                _ => unreachable!(),
            }
        }
    }))
}
