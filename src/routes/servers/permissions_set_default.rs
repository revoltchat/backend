use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use revolt_quark::{
    models::{server::PartialServer, Server, User},
    perms, Db, Error, Ref, Result,
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
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_manage_roles()
    {
        return Err(Error::NotFound);
    }

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
