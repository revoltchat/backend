use crate::database::*;
use crate::util::result::{Error, Result};
use crate::util::variables::MAX_BOT_COUNT;
use crate::util::regex::RE_USERNAME;

use mongodb::bson::{doc, to_document};
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use nanoid::nanoid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    name: String,
}

#[post("/create", data = "<info>")]
pub async fn create_bot(user: User, info: Json<Data>) -> Result<Value> {
    if user.bot.is_some() {
        return Err(Error::IsBot)
    }
    
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let bot_count = db_conn()
        .get_bot_count_owned_by_user(&user.id).await?;
    if bot_count as usize >= *MAX_BOT_COUNT {
        return Err(Error::ReachedMaximumBots)
    }

    let id = Ulid::new().to_string();
    let token = nanoid!(64);
    let bot = Bot {
        id: id.clone(),
        owner: user.id.clone(),
        token,
        public: false,
        interactions_url: None
    };

    if User::is_username_taken(&info.name).await? {
        return Err(Error::UsernameTaken);
    }

    db_conn().add_bot_user(&id, &info.name, &user.id).await?;
    db_conn().add_bot(&bot).await?;

    Ok(json!(bot))
}
