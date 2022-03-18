use revolt_quark::{
    bson::DateTime,
    models::message::{PartialMessage, SendableEmbed},
    models::{message::Content, Message, User},
    types::january::Embed,
    DateTimeContainer, Db, Error, Ref, Result,
};

use chrono::Utc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 2000))]
    content: Option<String>,
    #[validate(length(min = 0, max = 10))]
    embeds: Option<Vec<SendableEmbed>>,
}

#[patch("/<target>/messages/<msg>", data = "<edit>")]
pub async fn req(
    db: &Db,
    user: User,
    target: String,
    msg: Ref,
    edit: Json<Data>,
) -> Result<Json<Message>> {
    let edit = edit.into_inner();
    edit.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut message = msg.as_message(db).await?;
    if message.channel != target {
        return Err(Error::NotFound);
    }

    if message.author != user.id {
        return Err(Error::CannotEditMessage);
    }

    message.edited = Some(DateTimeContainer(DateTime::from_chrono(Utc::now())));
    let mut partial = PartialMessage {
        edited: message.edited,
        ..Default::default()
    };

    // 1. Handle content update
    if let Some(content) = edit.content {
        partial.content = Some(Content::Text(content));
    }

    // 2. Clear any auto generated embeds
    let mut new_embeds: Vec<Embed> = vec![];
    if let Some(embeds) = &message.embeds {
        for embed in embeds {
            if let Embed::Text(embed) = embed {
                new_embeds.push(Embed::Text(embed.clone()))
            }
        }
    }

    // 3. Replace if we are given new embeds
    if let Some(embeds) = edit.embeds {
        new_embeds.clear();

        for embed in embeds {
            new_embeds.push(embed.clone().into_embed(db, message.id.clone()).await?);
        }
    }

    partial.embeds = message.embeds.clone();

    message.update(db, partial).await?;
    Ok(Json(message))
}
