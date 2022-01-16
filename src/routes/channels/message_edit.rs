use crate::database::*;
use crate::util::result::{Error, Result, EmptyResponse};
use crate::routes::channels::message_send::SendableEmbed;

use chrono::Utc;
use mongodb::bson::{Bson, doc, to_document};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 2000))]
    content: Option<String>,
    #[validate(length(min = 0, max = 10))]
    embeds: Option<Vec<SendableEmbed>>
}

#[patch("/<target>/messages/<msg>", data = "<edit>")]
pub async fn req(user: User, target: Ref, msg: Ref, edit: Json<Data>) -> Result<EmptyResponse> {
    edit.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.fetch_channel().await?;
    channel.has_messaging()?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let mut message = msg.fetch_message(&channel).await?;
    if message.author != user.id {
        Err(Error::CannotEditMessage)?
    }

    let edited = Utc::now();
    let mut set = doc! { "edited": Bson::DateTime(edited) };
    let mut unset = doc! {};
    let mut update = json!({ "edited": Bson::DateTime(edited) });

    if let Some(new_content) = &edit.content {
        set.insert("content", new_content.clone());
        update.as_object_mut().unwrap().insert("content".to_string(), json!(new_content.clone()));
        message.content = Content::Text(new_content.clone());
    }

    let mut new_embeds: Vec<Embed> = if let Some(embeds) = message.embeds.clone() {
        embeds
        .into_iter()
        .filter(|e| matches!(e, Embed::Image(_) | Embed::Text(_)))
        .collect()
    } else {
        vec![]
    };

    if let Some(edited_embeds) = &edit.embeds {
        new_embeds.clear();
        for embed in edited_embeds.clone() {
            new_embeds.push(embed.into_embed(message.id.clone()).await?)
        }
    };

    set.insert("embeds", new_embeds
        .iter()
        .map(|e| to_document(e).unwrap())
        .collect::<Vec<_>>()
    );

    if edit.embeds.as_ref().map(|v| v.is_empty()).unwrap_or_default() {
        set.remove("embeds");
        unset.insert("embeds", 1u32);
    };

    get_collection("messages")
        .update_one(
            doc! {
                "_id": &message.id
            },
            doc! {
                "$set": set,
                "$unset": unset
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "message",
        })?;

    message.publish_update(update).await?;
    Ok(EmptyResponse {})
}
