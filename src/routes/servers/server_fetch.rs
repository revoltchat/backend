use revolt_quark::{
    models::{Server, User},
    perms, Db, Error, Ref, Result,
};
use rocket::serde::json::Json;

#[get("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Server>> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc(db)
        .await
        .can_view_channel()
    {
        return Err(Error::NotFound);
    }

    Ok(Json(server))
}
