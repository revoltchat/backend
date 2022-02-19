use revolt_quark::{
    models::{server::Role, User},
    perms, Db, Error, Ref, Result, DEFAULT_PERMISSION_CHANNEL_SERVER, DEFAULT_SERVER_PERMISSION,
};

use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: String,
}

#[post("/<target>/roles", data = "<data>")]
pub async fn req(db: &Db, user: User, target: Ref, data: Json<Data>) -> Result<Value> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_manage_roles()
    {
        return Err(Error::NotFound);
    }

    let role = Role {
        name: data.name,
        permissions: (
            *DEFAULT_SERVER_PERMISSION as i32,
            *DEFAULT_PERMISSION_CHANNEL_SERVER as i32,
        ),
        ..Default::default()
    };

    Ok(json!({
        "id": role.create(db, &server.id).await?
    }))
}
