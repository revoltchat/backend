use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result};

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
) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_manage_roles()
    {
        return Err(Error::NotFound);
    }

    db.update_role_permission(
        &server.id,
        &role_id,
        &(
            data.permissions.server as i32,
            data.permissions.channel as i32,
        ),
    )
    .await
    .map(|_| EmptyResponse)
}
