use revolt_database::{util::reference::Reference, Channel, Database, Invite};
use revolt_models::v0;
use revolt_result::Result;
use rocket::{serde::json::Json, State};

/// # Fetch Invite
///
/// Fetch an invite by its id.
#[openapi(tag = "Invites")]
#[get("/<target>")]
pub async fn fetch(db: &State<Database>, target: Reference) -> Result<Json<v0::InviteResponse>> {
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

                    v0::InviteResponse::Server {
                        code: target.id,
                        member_count: db.fetch_member_count(&server.id).await? as i64,
                        server_id: server.id,
                        server_name: server.name,
                        server_icon: server.icon.map(|f| f.into()),
                        server_banner: server.banner.map(|f| f.into()),
                        server_flags: server.flags,
                        channel_id: id,
                        channel_name: name,
                        channel_description: description,
                        user_name: user.username,
                        user_avatar: user.avatar.map(|f| f.into()),
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
                } => v0::InviteResponse::Group {
                    code: target.id,
                    channel_id: id,
                    channel_name: name,
                    channel_description: description,
                    user_name: user.username,
                    user_avatar: user.avatar.map(|f| f.into()),
                },
                _ => unreachable!(),
            }
        }
    }))
}
