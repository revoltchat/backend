use revolt_quark::{
    models::{Channel, User, Member, server_member::MemberCompositeKey},
    perms, ChannelPermission, Db, EmptyResponse, Error, Ref, Result, ServerPermission,
};

use rocket::serde::json::Json;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Destination {
    Server { server: String },
    Group { group: String },
}

#[post("/<target>/invite", data = "<dest>")]
pub async fn invite_bot(
    db: &Db,
    user: User,
    target: Ref,
    dest: Json<Destination>,
) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = target.as_bot(db).await?;
    if !bot.public && bot.owner != user.id {
        return Err(Error::BotIsPrivate);
    }

    match dest.into_inner() {
        Destination::Server { server } => {
            let server = db.fetch_server(&server).await?;
            if !perms(&user)
                .server(&server)
                .calc_server(db)
                .await
                .get_manage_server()
            {
                return Err(Error::MissingPermission {
                    permission: ServerPermission::ManageServer as i32,
                });
            }

            let member = Member {
                id: MemberCompositeKey {
                    server: server.id,
                    user: bot.id
                },
                ..Default::default()
            };

            db.insert_member(&member)
                .await
                .map(|_| EmptyResponse)
        }
        Destination::Group { group } => {
            let channel = db.fetch_channel(&group).await?;
            if !perms(&user)
                .channel(&channel)
                .calc_channel(db)
                .await
                .get_invite_others()
            {
                return Err(Error::MissingPermission {
                    permission: ChannelPermission::InviteOthers as i32,
                });
            }

            if let Channel::Group { id, .. } = channel {
                db.add_user_to_group(&id, &bot.id)
                    .await
                    .map(|_| EmptyResponse)
            } else {
                Err(Error::InvalidOperation)
            }
        }
    }
}
