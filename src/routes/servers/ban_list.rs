use revolt_quark::models::{File, ServerBan, User};
use revolt_quark::{perms, Db, Error, Permission, Ref, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BannedUser {
    _id: String,
    username: String,
    avatar: Option<File>,
}

#[get("/<target>/bans")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<ServerBan>>> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc(db)
        .await
        .can_ban_members()
    {
        return Error::from_permission(Permission::BanMembers);
    }

    db.fetch_bans(&server.id).await.map(Json)
}
