use revolt_quark::{
    models::{ServerBan, User},
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 1024))]
    reason: Option<String>,
}

#[put("/<server>/bans/<target>", data = "<data>")]
pub async fn req(
    db: &Db,
    user: User,
    server: Ref,
    target: Ref,
    data: Json<Data>,
) -> Result<Json<ServerBan>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let server = server.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::BanMembers)
        .await?;

    let member = target.as_member(db, &server.id).await?;
    // ! FIXME_PERMISSIONS

    let ban = ServerBan {
        id: member.id,
        reason: data.reason,
    };

    db.insert_ban(&ban).await?;
    Ok(Json(ban))
}
