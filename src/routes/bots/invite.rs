use crate::database::*;
use crate::util::result::{Error, Result, EmptyResponse};

use rocket::serde::json::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerId {
    server: String
}

#[derive(Deserialize)]
pub struct GroupId {
    group: String
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Destination {
    Server(ServerId),
    Group(GroupId)
}

#[post("/<target>/invite", data = "<dest>")]
pub async fn invite_bot(user: User, target: Ref, dest: Json<Destination>) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    let bot = target.fetch_bot().await?;

    if !bot.public {
        if bot.owner != user.id {
            return Err(Error::BotIsPrivate);
        }
    }

    match dest.into_inner() {
        Destination::Server(ServerId { server }) => {
            let server = Ref::from(server)?.fetch_server().await?;

            let perm = permissions::PermissionCalculator::new(&user)
                .with_server(&server)
                .for_server()
                .await?;

            if !perm.get_manage_server() {
                Err(Error::MissingPermission)?
            }

            server.join_member(&bot.id).await?;
            Ok(EmptyResponse {})
        }
        Destination::Group(GroupId { group }) => {
            let channel = Ref::from(group)?.fetch_channel().await?;

            let perm = permissions::PermissionCalculator::new(&user)
                .with_channel(&channel)
                .for_channel()
                .await?;

            if !perm.get_invite_others() {
                Err(Error::MissingPermission)?
            }

            channel.add_to_group(bot.id, user.id).await?;
            Ok(EmptyResponse {})
        }
    }
}
