use revolt_quark::models::{File, User, ServerBan};
use revolt_quark::{perms, Db, Error, Ref, Result, ServerPermission};

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
        .calc_server(db)
        .await
        .get_ban_members()
    {
        return Err(Error::MissingPermission {
            permission: ServerPermission::BanMembers as i32,
        });
    }

    db.fetch_bans(&server.id).await.map(Json)
}
