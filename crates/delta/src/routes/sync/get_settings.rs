use revolt_database::{Database, User};
use revolt_models::v0;
use revolt_result::Result;
use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Settings
///
/// Fetch settings from server filtered by keys.
///
/// This will return an object with the requested keys, each value is a tuple of `(timestamp, value)`, the value is the previously uploaded data.
#[openapi(tag = "Sync")]
#[post("/settings/fetch", data = "<options>")]
pub async fn fetch(
    db: &State<Database>,
    user: User,
    options: Json<v0::OptionsFetchSettings>,
) -> Result<Json<v0::UserSettings>> {
    db.fetch_user_settings(&user.id, &options.into_inner().keys)
        .await
        .map(Json)
}
