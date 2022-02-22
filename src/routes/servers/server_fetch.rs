use revolt_quark::{
    models::{Server, User},
    perms, Db, Ref, Result,
};
use rocket::serde::json::Json;

#[get("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Server>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    Ok(Json(server))
}
