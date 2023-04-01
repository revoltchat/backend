use revolt_quark::{
    models::message::{PartialMessage, SendableEmbed},
    models::{Message, User},
    types::january::Embed,
    Db, Error, Ref, Result, Timestamp,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Message Details
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataEditMessage {
    /// New message content
    #[validate(length(min = 1, max = 2000))]
    content: Option<String>,
    /// Embeds to include in the message
    #[validate(length(min = 0, max = 10))]
    embeds: Option<Vec<SendableEmbed>>,
}

/// # Edit Message
///
/// Edits a message that you've previously sent.
#[openapi(tag = "Messaging")]
#[patch("/<target>/messages/<msg>", data = "<edit>")]
pub async fn req(
    db: &Db,
    user: User,
    target: String,
    msg: Ref,
    edit: Json<DataEditMessage>,
) -> Result<Json<Message>> {
    let edit = edit.into_inner();
    edit.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    Message::validate_sum(&edit.content, &edit.embeds)?;

    let mut message = msg.as_message(db).await?;
    if message.channel != target {
        return Err(Error::NotFound);
    }

    if message.author != user.id {
        return Err(Error::CannotEditMessage);
    }

    message.edited = Some(Timestamp::now_utc());
    let mut partial = PartialMessage {
        edited: message.edited,
        ..Default::default()
    };

    // 1. Handle content update
    if let Some(content) = &edit.content {
        partial.content = Some(content.clone());
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

    partial.embeds = Some(new_embeds);

    message.update(db, partial).await?;

    // Queue up a task for processing embeds
    if let Some(content) = edit.content {
        revolt_quark::tasks::process_embeds::queue(
            message.channel.to_string(),
            message.id.to_string(),
            content,
        )
        .await;
    }

    Ok(Json(message))
}
