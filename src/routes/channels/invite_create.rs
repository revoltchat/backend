use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use nanoid::nanoid;
use rocket_contrib::json::JsonValue;

lazy_static! {
    static ref ALPHABET: [char; 54] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
        'J', 'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z'
    ];
}

#[post("/<target>/invites")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let target = target.fetch_channel().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_invite_others() {
        return Err(Error::MissingPermission);
    }

    let code = nanoid!(8, &*ALPHABET);
    match &target {
        Channel::Group { .. } => {
            unimplemented!()
        }
        Channel::TextChannel { id, server, .. } => {
            Invite::Server {
                code: code.clone(),
                creator: user.id,
                server: server.clone(),
                channel: id.clone(),
            }
            .save()
            .await?;

            Ok(json!({ "code": code }))
        }
        _ => Err(Error::InvalidOperation),
    }
}
