use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{Server, User},
    perms, Db, Override, Permission, Ref, Result,
};

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: Override,
}

#[put("/<target>/permissions/<role_id>", data = "<data>", rank = 2)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    role_id: String,
    data: Json<Data>,
) -> Result<Json<Server>> {
    let data = data.into_inner();

    let mut server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManagePermissions)
        .await?;

    // ! FIXME_PERMISSIONS

    server
        .set_role_permission(db, &role_id, data.permissions.into())
        .await?;

    Ok(Json(server))
}
