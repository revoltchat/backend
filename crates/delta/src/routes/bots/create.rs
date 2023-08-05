use nanoid::nanoid;

use revolt_database::{Bot, BotInformation, Database, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use rocket::serde::json::Json;
use rocket::State;
use ulid::Ulid;
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
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    }

    let info = info.into_inner();
    info.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    // TODO: config
    let max_bot_count = 5;
    if db.get_number_of_bots_by_user(&user.id).await? >= max_bot_count {
        return Err(create_error!(ReachedMaximumBots));
    }

    let id = Ulid::new().to_string();
    let username = User::validate_username(info.name)?;
    let bot_user = User {
        id: id.clone(),
        discriminator: User::find_discriminator(db, &username, None).await?,
        username,
        bot: Some(BotInformation {
            owner: user.id.clone(),
        }),
        ..Default::default()
    };

    let bot = Bot {
        id,
        owner: user.id,
        token: nanoid!(64),
        ..Default::default()
    };

    db.insert_user(&bot_user).await?;
    db.insert_bot(&bot).await?;
    Ok(Json(bot.into()))
}
