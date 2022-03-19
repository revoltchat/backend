use revolt_quark::{
    models::{Server, User},
    perms, Db, Ref, Result,
};
use rocket::serde::json::Json;

/// # Fetch Server
///
/// Fetch a server by its id.
#[openapi(tag = "Server Information")]
#[get("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Server>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    Ok(Json(server))
}
