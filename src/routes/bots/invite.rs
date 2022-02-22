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
            if !perms(&user)
                .server(&server)
                .calc(db)
                .await
                .can_manage_server()
            {
                return Error::from_permission(Permission::ManageServer);
            }

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
            if !perms(&user)
                .channel(&channel)
                .calc(db)
                .await
                .can_invite_others()
            {
                return Error::from_permission(Permission::InviteOthers);
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
