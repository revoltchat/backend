use revolt_database::{Bot, Database, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use rocket::serde::json::Json;
use rocket::State;
use validator::Validate;

/// # Create Bot
///
/// Create a new Revolt bot.
#[openapi(tag = "Bots")]
#[post("/create", data = "<info>")]
pub async fn create_bot(
    db: &State<Database>,
    user: User,
    info: Json<v0::DataCreateBot>,
) -> Result<Json<v0::Bot>> {
    let info = info.into_inner();
    info.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let bot = Bot::create(db, info.name, &user, None).await?;
    Ok(Json(bot.into()))
}
