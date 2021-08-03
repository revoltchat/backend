use crate::database::*;
use crate::util::result::Result;

use rocket::serde::json::Value;

#[post("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
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
