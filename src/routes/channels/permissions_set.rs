use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{Channel, User},
    perms, ChannelPermission, Db, EmptyResponse, Error, Ref, Result,
};

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: u32,
}

#[put("/<target>/permissions/<role>", data = "<data>", rank = 2)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role: String,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
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

    match channel {
        Channel::TextChannel { id, .. } | Channel::VoiceChannel { id, .. } => {
            db.set_channel_role_permission(&id, &role, data.permissions)
                .await?;
        }
        _ => return Err(Error::InvalidOperation),
    }

    Ok(EmptyResponse)
}
