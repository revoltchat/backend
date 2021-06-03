use crate::database::*;
use crate::util::result::Result;

use rocket_contrib::json::JsonValue;

#[post("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let target = target.fetch_invite().await?;

    match target {
        Invite::Server {
            channel,
            ..
        } => {
            let channel = Ref::from_unchecked(channel).fetch_channel().await?;
            let server = if let Channel::TextChannel { server, .. } = &channel {
                Ref::from_unchecked(server.clone()).fetch_server().await?
            } else {
                unreachable!()
            };

            server.join_member(&user.id).await?;

            Ok(json!({
                "channel": channel,
                "server": server
            }))
        }
        _ => unreachable!(),
    }
}
