use revolt_quark::{
    models::{Invite, User},
    perms, Db, Error, Permission, Ref, Result,
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
        .calc(db)
        .await
        .can_manage_server()
    {
        return Error::from_permission(Permission::ManageServer);
    }

    db.fetch_invites_for_server(&server.id).await.map(Json)
}
