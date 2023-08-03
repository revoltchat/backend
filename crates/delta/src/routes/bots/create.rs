use crate::util::regex::RE_USERNAME;

use nanoid::nanoid;

use rocket::serde::json::Json;
use serde::Deserialize;
use ulid::Ulid;
use validator::Validate;

/// # Create Bot
///
/// Create a new Revolt bot.
#[openapi(tag = "Bots")]
#[post("/create", data = "<info>")]
pub async fn create_bot(db: &Db, user: User, info: Json<DataCreateBot>) -> Result<Json<Bot>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if db.get_number_of_bots_by_user(&user.id).await? >= *MAX_BOT_COUNT {
        return Err(Error::ReachedMaximumBots);
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
    Ok(Json(bot))
}
