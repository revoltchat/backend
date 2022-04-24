use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Permission, Ref, Result};

use rocket::serde::json::Json;
use serde::Deserialize;

/// # Invite Destination
#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum InviteBotDestination {
    /// Invite to a server
    Server {
        /// Server Id
        server: String,
    },
    /// Invite to a group
    Group {
        /// Group Id
        group: String,
    },
}

/// # Invite Bot
///
/// Invite a bot to a server or group by its id.`
#[openapi(tag = "Bots")]
#[post("/<target>/invite", data = "<dest>")]
pub async fn invite_bot(
    db: &Db,
    user: User,
    target: Ref,
    dest: Json<InviteBotDestination>,
) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = target.as_bot(db).await?;
    if !bot.public && bot.owner != user.id {
        return Err(Error::BotIsPrivate);
    }

    match dest.into_inner() {
        InviteBotDestination::Server { server } => {
            let server = db.fetch_server(&server).await?;

            perms(&user)
                .server(&server)
                .throw_permission(db, Permission::ManageServer)
                .await?;

            let user = db.fetch_user(&bot.id).await?;
            server
                .create_member(db, user, None)
                .await
                .map(|_| EmptyResponse)
        }
        InviteBotDestination::Group { group } => {
            let mut channel = db.fetch_channel(&group).await?;

            perms(&user)
                .channel(&channel)
                .throw_permission_and_view_channel(db, Permission::InviteOthers)
                .await?;

            channel
                .add_user_to_group(db, &bot.id, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
    }
}
