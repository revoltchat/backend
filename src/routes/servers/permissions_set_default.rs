use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{server::PartialServer, Server, User},
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

#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(db: &Db, user: User, target: Ref, data: Json<Data>) -> Result<Json<Server>> {
    let data = data.into_inner();

    let mut server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManagePermissions)
        .await?;

    // ! FIXME_PERMISSIONS

    server
        .update(
            db,
            PartialServer {
                default_permissions: Some((
                    data.permissions.server as i32,
                    data.permissions.channel as i32,
                )),
                ..Default::default()
            },
            vec![],
        )
        .await?;

    Ok(Json(server))
}
