use revolt_quark::{
    models::{Invite, User},
    perms, Db, Error, Ref, Result, ServerPermission,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInvite {
    #[serde(rename = "_id")]
    code: String,
    creator: String,
    channel: String,
}

#[get("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<Invite>>> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_manage_server()
    {
        return Err(Error::MissingPermission {
            permission: ServerPermission::ManageServer as i32,
        });
    }

    db.fetch_invites_for_server(&server.id).await.map(Json)
}
