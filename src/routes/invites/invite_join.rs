use revolt_quark::{Error, Result, Ref, models::{User, Invite}, Db};

use rocket::serde::json::Value;

use crate::util::variables::MAX_SERVER_COUNT;

#[post("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }

    if !user.can_acquire_server(db).await? {
        return Err(Error::TooManyServers {
            max: *MAX_SERVER_COUNT
        })
    }

    let invite = target.as_invite(db).await?;
    match &invite {
        Invite::Server { channel, server, .. } => {
            let server = db.fetch_server(server).await?;
            if db.fetch_ban(&server.id, &user.id).await.is_ok() {
                return Err(Error::Banned)
            }

            let channel = db.fetch_channel(channel).await?;
            db.insert_member(&server.id, &user.id).await?;
            
            Ok(json!({
                "type": "Server",
                "channel": channel,
                "server": server
            }))
        }
        _ => unreachable!()
    }
}
