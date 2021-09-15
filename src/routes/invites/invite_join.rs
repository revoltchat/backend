use crate::database::*;
use crate::util::result::{Error, Result};
use crate::util::variables::MAX_SERVER_COUNT;

use rocket::serde::json::Value;

#[post("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }

    if !User::can_acquire_server(&user.id).await? {
        Err(Error::TooManyServers {
            max: *MAX_SERVER_COUNT,
        })?
    }
    
    let target = target.fetch_invite().await?;

    match target {
        Invite::Server { channel, .. } => {
            let channel = Ref::from_unchecked(channel).fetch_channel().await?;
            let server = match &channel {
                Channel::TextChannel { server, .. }
                | Channel::VoiceChannel { server, .. } => {
                    Ref::from_unchecked(server.clone()).fetch_server().await?
                }
                _ => unreachable!()
            };

            server.join_member(&user.id).await?;

            Ok(json!({
                "type": "Server",
                "channel": channel,
                "server": server
            }))
        }
        _ => unreachable!(),
    }
}
