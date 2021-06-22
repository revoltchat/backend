use crate::database::*;
use crate::util::result::{Error, Result};

use chrono::Utc;
use mongodb::bson::{doc, Bson, DateTime, Document};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 2000))]
    content: String,
}

#[patch("/<target>/messages/<msg>", data = "<edit>")]
pub async fn req(user: User, target: Ref, msg: Ref, edit: Json<Data>) -> Result<()> {
    edit.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.fetch_channel().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&channel)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let message = msg.fetch_message(&channel).await?;
    if message.author != user.id {
        Err(Error::CannotEditMessage)?
    }

    let edited = Utc::now();
    let mut set = doc! {
        "content": &edit.content,
        "edited": Bson::DateTime(edited)
    };

    let mut update = json!({ "content": edit.content, "edited": DateTime(edited) });

    if let Some(embeds) = &message.embeds {
        let new_embeds: Vec<Document> = vec![];

        for embed in embeds {
            match embed {
                Embed::Website(_) | Embed::Image(_) | Embed::None => {} // Otherwise push to new_embeds.
            }
        }

        let obj = update.as_object_mut().unwrap();
        obj.insert("embeds".to_string(), json!(new_embeds).0);
        set.insert("embeds", new_embeds);
    }

    get_collection("messages")
        .update_one(
            doc! {
                "_id": &message.id
            },
            doc! {
                "$set": set
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "message",
        })?;

    message.publish_update(update).await
}
