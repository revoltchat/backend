use revolt_quark::models::{File, ServerBan, User};
use revolt_quark::{perms, Db, Permission, Ref, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

/// # Banned User
///
/// Just enoguh user information to list bans.
#[derive(Serialize, Deserialize, JsonSchema)]
struct BannedUser {
    /// Id of the banned user
    #[serde(rename = "_id")]
    pub id: String,
    /// Username of the banned user
    pub username: String,
    /// Avatar of the banned user
    pub avatar: Option<File>,
}

/// # Ban List Result
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct BanListResult {
    /// Users objects
    users: Vec<BannedUser>,
    /// Ban objects
    bans: Vec<ServerBan>,
}

impl From<User> for BannedUser {
    fn from(user: User) -> Self {
        BannedUser {
            id: user.id,
            username: user.username,
            avatar: user.avatar,
        }
    }
}

/// # Fetch Bans
///
/// Fetch all bans on a server.
#[openapi(tag = "Server Members")]
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
        .map(|x| x.into())
        .collect();

    Ok(Json(BanListResult { users, bans }))
}
