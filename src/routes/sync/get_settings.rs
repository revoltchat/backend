use revolt_quark::{
    models::{User, UserSettings},
    Db, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

/// # Fetch Options
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OptionsFetchSettings {
    /// Keys to fetch
    keys: Vec<String>,
}

/// # Fetch Settings
///
/// Fetch settings from server filtered by keys.
///
/// This will return an object with the requested keys, each value is a tuple of `(timestamp, value)`, the value is the previously uploaded data.
#[openapi(tag = "Sync")]
#[post("/settings/fetch", data = "<options>")]
pub async fn req(
    db: &Db,
    user: User,
    options: Json<OptionsFetchSettings>,
) -> Result<Json<UserSettings>> {
    db.fetch_user_settings(&user.id, &options.into_inner().keys)
        .await
        .map(Json)
}
