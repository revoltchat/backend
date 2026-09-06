use revolt_database::{
    util::reference::Reference, Channel, Database, Invite, InviteAttribution, Member, User, AMQP,
};
use revolt_models::v0::{self, InviteJoinResponse};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Join Invite
///
/// Join an invite by its ID
#[openapi(tag = "Invites")]
#[post("/<target>")]
pub async fn join(
    db: &State<Database>,
    amqp: &State<AMQP>,
    user: User,
    target: Reference<'_>,
) -> Result<Json<v0::InviteJoinResponse>> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    user.can_acquire_server(db).await?;

    let invite = target.as_invite(db).await?;
    match &invite {
        Invite::Server {
            server,
            creator,
            code,
            max_uses,
            uses,
            ..
        } => {
            // `max_uses` absent means unlimited, which is what every invite
            // that predates this field carries - including the migration link.
            // Enforcement can therefore only bite on a code that explicitly
            // asked for a limit.
            if let Some(limit) = max_uses {
                if uses >= limit {
                    return Err(create_error!(InviteExhausted));
                }
            }

            let server = db.fetch_server(server).await?;
            // Record BOTH who invited them and which code was used. The code is
            // the attribution key - it is what tells a member's personal invite
            // apart from the website, a social post, or the migration link -
            // and neither survives on a system message in a channel.
            let (_, channels) = Member::create(
                db,
                &server,
                &user,
                None,
                Some(InviteAttribution {
                    by: creator.clone(),
                    code: code.clone(),
                }),
            )
            .await?;

            // Count the use only after the join actually succeeded, so a
            // failed join does not burn someone's one-shot code.
            if let Err(err) = db.increment_invite_uses(code).await {
                revolt_config::capture_error(&err);
            }

            // Registration is validated against authifier's OWN invite store,
            // which our code mirrors into on creation. Deleting the mirror when
            // a code is spent is what makes the limit hold for ACCOUNT CREATION
            // as well as for joining - without forking the authifier crate.
            if let Some(limit) = max_uses {
                if uses + 1 >= *limit {
                    // NOT `authifier.database` - the crate has no delete, and
                    // measured against 1.0.16 its create_account never checks
                    // the `used` flag it sets, so marking a code used does not
                    // stop it registering more accounts. Removing the mirror
                    // row is what actually closes registration.
                    db.delete_authifier_invite_mirror(code).await?;
                }
            }

            Ok(Json(InviteJoinResponse::Server {
                channels: channels.into_iter().map(|c| c.into()).collect(),
                server: server.into(),
            }))
        }
        Invite::Group {
            channel, creator, ..
        } => {
            let mut channel = db.fetch_channel(channel).await?;
            channel.add_user_to_group(db, amqp, &user, creator).await?;
            if let Channel::Group { recipients, .. } = &channel {
                Ok(Json(InviteJoinResponse::Group {
                    users: User::fetch_many_ids_as_mutuals(db, &user, recipients).await?,
                    channel: channel.into(),
                }))
            } else {
                unreachable!()
            }
        }
    }
}
