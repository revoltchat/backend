use crate::util::regex::RE_USERNAME;

use nanoid::nanoid;
use revolt_quark::{
    models::{user::BotInformation, Bot, User},
    variables::delta::MAX_BOT_COUNT,
    Db, Error, Result,
};

use rocket::serde::json::Json;
use serde::Deserialize;
use ulid::Ulid;
use validator::Validate;

/// # Bot Details
#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataCreateBot {
    /// Bot username
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    name: String,
}

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

    if db.is_username_taken(&info.name).await? {
        return Err(Error::UsernameTaken);
    }

    let id = Ulid::new().to_string();
    let bot_user = User {
        id: id.clone(),
        username: info.name.trim().to_string(),
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
