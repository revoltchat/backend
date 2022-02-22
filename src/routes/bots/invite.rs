use revolt_quark::{
    models::{server_member::MemberCompositeKey, Channel, Member, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
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

            perms(&user)
                .server(&server)
                .throw_permission(db, Permission::ManageServer)
                .await?;

            let member = Member {
                id: MemberCompositeKey {
                    server: server.id,
                    user: bot.id,
                },
                ..Default::default()
            };

            db.insert_member(&member).await.map(|_| EmptyResponse)
        }
        Destination::Group { group } => {
            let channel = db.fetch_channel(&group).await?;

            perms(&user)
                .channel(&channel)
                .throw_permission_and_view_channel(db, Permission::InviteOthers)
                .await?;

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
