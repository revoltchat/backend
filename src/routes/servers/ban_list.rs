use revolt_quark::models::{File, ServerBan, User};
use revolt_quark::{perms, Db, Permission, Ref, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BannedUser {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub avatar: Option<File>,
}

#[derive(Serialize, Deserialize)]
pub struct BanListResult {
    users: Vec<BannedUser>,
    bans: Vec<ServerBan>,
}

#[get("/<target>/bans")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<BanListResult>> {
    let server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::BanMembers)
        .await?;

    let bans = db.fetch_bans(&server.id).await?;
    let users = db
        .fetch_users(
            &bans
                .iter()
                .map(|x| &x.id.user)
                .cloned()
                .collect::<Vec<String>>(),
        )
        .await?
        .into_iter()
        .map(|x| BannedUser {
            id: x.id,
            username: x.username,
            avatar: x.avatar,
        })
        .collect();

    Ok(Json(BanListResult { users, bans }))
}
