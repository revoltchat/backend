use revolt_quark::{
    models::{server_member::MemberCompositeKey, Invite, Member, User},
    Db, Error, Ref, Result,
};

use rocket::serde::json::Value;

use crate::util::variables::MAX_SERVER_COUNT;

#[post("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Value> {
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

            Ok(json!({
                "type": "Server",
                "channel": channel,
                "server": server
            }))
        }
        _ => unreachable!(),
    }
}
