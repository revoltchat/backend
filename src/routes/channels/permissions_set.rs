use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{Channel, User},
    perms, ChannelPermission, Db, Error, Ref, Result,
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
) -> Result<Json<Channel>> {
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

    channel
        .set_role_permission(db, &role, data.permissions)
        .await?;

    Ok(Json(channel))
}
