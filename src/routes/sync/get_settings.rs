use revolt_quark::{
    models::{User, UserSettings},
    Db, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Options {
    keys: Vec<String>,
}

#[post("/settings/fetch", data = "<options>")]
pub async fn req(db: &Db, user: User, options: Json<Options>) -> Result<Json<UserSettings>> {
    db.fetch_user_settings(&user.id, &options.into_inner().keys)
        .await
        .map(Json)
}
