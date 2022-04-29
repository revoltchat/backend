use crate::util::regex::RE_USERNAME;

use revolt_quark::{
    models::{
        bot::{FieldsBot, PartialBot},
        Bot, User,
    },
    Db, Error, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Bot Details
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataEditBot {
    /// Bot username
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Whether the bot can be added by anyone
    public: Option<bool>,
    /// Whether analytics should be gathered for this bot
    ///
    /// Must be enabled in order to show up on [Revolt Discover](https://rvlt.gg).
    analytics: Option<bool>,
    /// Interactions URL
    #[validate(length(min = 1, max = 2048))]
    interactions_url: Option<String>,
    /// Fields to remove from bot object
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsBot>>,
}

/// # Edit Bot
///
/// Edit bot details by its id.
#[openapi(tag = "Bots")]
#[patch("/<target>", data = "<data>")]
pub async fn edit_bot(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataEditBot>,
) -> Result<Json<Bot>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    if let Some(name) = data.name {
        if db.is_username_taken(&name).await? {
            return Err(Error::UsernameTaken);
        }

        let mut user = db.fetch_user(&bot.id).await?;
        user.update_username(db, name).await?;
    }

    if data.public.is_none()
        && data.analytics.is_none()
        && data.interactions_url.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(bot));
    }

    let DataEditBot {
        public,
        analytics,
        interactions_url,
        remove,
        ..
    } = data;

    let mut partial = PartialBot {
        public,
        analytics,
        interactions_url,
        ..Default::default()
    };

    if let Some(remove) = &remove {
        for field in remove {
            bot.remove(field);
        }

        if remove.iter().any(|x| x == &FieldsBot::Token) {
            partial.token = Some(bot.token.clone());
        }
    }

    db.update_bot(&bot.id, &partial, remove.unwrap_or_default())
        .await?;

    bot.apply_options(partial);
    Ok(Json(bot))
}
