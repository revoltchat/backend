use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{Channel, User},
    perms, Db, Permission, Ref, Result,
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
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, Permission::ManagePermissions)
        .await?;

    // ! FIXME_PERMISSIONS

    channel
        .set_role_permission(db, &role, data.permissions)
        .await?;

    Ok(Json(channel))
}
