use revolt_database::util::permissions::DatabasePermissionQuery;
use revolt_database::Member;
use revolt_database::{util::reference::Reference, Database, User};
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::State;

use rocket::serde::json::Json;
use rocket_empty::EmptyResponse;
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
    db: &State<Database>,
    user: User,
    target: Reference,
    dest: Json<InviteBotDestination>,
) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    let bot = target.as_bot(db).await?;
    if !bot.public && bot.owner != user.id {
        return Err(create_error!(BotIsPrivate));
    }

    let bot_user = db.fetch_user(&bot.id).await?;

    match dest.into_inner() {
        InviteBotDestination::Server { server } => {
            let server = db.fetch_server(&server).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
            calculate_server_permissions(&mut query)
                .await
                .throw_if_lacking_channel_permission(ChannelPermission::ManageServer)?;

            Member::create(db, &server, &bot_user)
                .await
                .map(|_| EmptyResponse)
        }
        InviteBotDestination::Group { group } => {
            let mut channel = db.fetch_channel(&group).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
            calculate_server_permissions(&mut query)
                .await
                .throw_if_lacking_channel_permission(ChannelPermission::InviteOthers)?;

            channel
                .add_user_to_group(db, &bot_user, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
    }
}
