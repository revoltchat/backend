use revolt_quark::{
    models::{Invite, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;

/// # Fetch Invites
///
/// Fetch all server invites.
#[openapi(tag = "Server Members")]
#[get("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<Invite>>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManageServer)
        .await?;

    db.fetch_invites_for_server(&server.id).await.map(Json)
}
