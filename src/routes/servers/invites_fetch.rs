use revolt_quark::{
    models::{Invite, User},
    perms, Db, Permission, Ref, Result,
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
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManageServer)
        .await?;

    db.fetch_invites_for_server(&server.id).await.map(Json)
}
