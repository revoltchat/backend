use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{Server, User},
    perms, Db, Permission, Ref, Result,
};

#[derive(Serialize, Deserialize)]
pub struct Values {
    server: u32,
    channel: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: Values,
}

#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role_id: String,
    data: Json<Data>,
) -> Result<Json<Server>> {
    let mut server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManagePermissions)
        .await?;

    // ! FIXME_PERMISSIONS

    server
        .set_role_permission(
            db,
            &role_id,
            &(
                data.permissions.server as i32,
                data.permissions.channel as i32,
            ),
        )
        .await?;

    Ok(Json(server))
}
