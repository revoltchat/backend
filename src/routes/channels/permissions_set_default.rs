use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{channel::PartialChannel, Channel, User},
    perms, ChannelPermission, Db, Error, Ref, Result,
};

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: u32,
}

#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(db: &Db, user: User, target: Ref, data: Json<Data>) -> Result<Json<Channel>> {
    let mut channel = target.as_channel(db).await?;
    if !perms(&user)
        .channel(&channel)
        .calc_channel(db)
        .await
        .get_manage_channel()
    {
        return Err(Error::MissingPermission {
            permission: ChannelPermission::ManageChannel as i32,
        });
    }

    match &channel {
        Channel::Group { .. } => {
            channel
                .update(
                    db,
                    PartialChannel {
                        permissions: Some(data.permissions as i32),
                        ..Default::default()
                    },
                    vec![],
                )
                .await?;
        }
        Channel::TextChannel { .. } | Channel::VoiceChannel { .. } => {
            channel
                .update(
                    db,
                    PartialChannel {
                        default_permissions: Some(data.permissions as i32),
                        ..Default::default()
                    },
                    vec![],
                )
                .await?;
        }
        _ => return Err(Error::InvalidOperation),
    }

    Ok(Json(channel))
}
