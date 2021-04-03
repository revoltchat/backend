use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use mongodb::bson::{doc, to_document};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[validate(length(min = 0, max = 1024))]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[patch("/<target>", data = "<info>")]
pub async fn req(user: User, target: Ref, info: Json<Data>) -> Result<()> {
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if info.name.is_none() && info.description.is_none() {
        return Ok(());
    }

    let target = target.fetch_channel().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;

    if !perm.get_manage_channel() {
        Err(Error::MissingPermission)?
    }

    match &target {
        Channel::Group { id, .. } => {
            get_collection("channels")
            .update_one(
                doc! { "_id": &id },
                doc! { "$set": to_document(&info.0).map_err(|_| Error::DatabaseError { operation: "to_document", with: "data" })? },
                None
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "channel" })?;

            ClientboundNotification::ChannelUpdate {
                id: id.clone(),
                data: json!(info.0),
            }
            .publish(id.clone())
            .await
            .ok();

            if let Some(name) = &info.name {
                Message::create(
                    "00000000000000000000000000".to_string(),
                    id.clone(),
                    Content::SystemMessage(SystemMessage::ChannelRenamed {
                        name: name.clone(),
                        by: user.id,
                    }),
                )
                .publish(&target)
                .await
                .ok();
            }

            Ok(())
        }
        _ => Err(Error::InvalidOperation),
    }
}
