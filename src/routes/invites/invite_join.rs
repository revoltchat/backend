use revolt_quark::{
    models::{server_member::MemberCompositeKey, Channel, Invite, Member, Server, User},
    Db, Error, Ref, Result,
};

use rocket::serde::json::Json;
use serde::Serialize;

use crate::util::variables::MAX_SERVER_COUNT;

/// # Join Response
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum InviteJoinResponse {
    Server {
        /// Channel we are joining
        channel: Channel,
        /// Server we are joining
        server: Server,
    },
}

/// # Join Invite
///
/// Join an invite by its ID.
#[openapi(tag = "Invites")]
#[post("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<InviteJoinResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    if !user.can_acquire_server(db).await? {
        return Err(Error::TooManyServers {
            max: *MAX_SERVER_COUNT,
        });
    }

    let invite = target.as_invite(db).await?;
    match &invite {
        Invite::Server {
            channel, server, ..
        } => {
            let server = db.fetch_server(server).await?;
            let channel = db.fetch_channel(channel).await?;
            let member = Member {
                id: MemberCompositeKey {
                    server: server.id.clone(),
                    user: user.id.clone(),
                },
                ..Default::default()
            };

            member.create(db).await?;

            Ok(Json(InviteJoinResponse::Server { channel, server }))
        }
        _ => unreachable!(),
    }
}
