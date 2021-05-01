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
    #[validate(length(min = 1, max = 128))]
    icon: Option<String>,
}

#[patch("/<target>", data = "<info>")]
pub async fn req(user: User, target: Ref, info: Json<Data>) -> Result<()> {
    let info = info.into_inner();
    info.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if info.name.is_none() && info.description.is_none() && info.icon.is_none() {
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
        Channel::Group { id, icon, .. } => {
            let mut set = doc! {};
            if let Some(name) = &info.name {
                set.insert("name", name);
            }

            if let Some(description) = info.description {
                set.insert("description", description);
            }

            let mut remove_icon = false;
            if let Some(attachment_id) = info.icon {
                let attachment = File::find_and_use(&attachment_id, "icons", "object", &user.id).await?;
                set.insert(
                    "icon",
                    to_document(&attachment).map_err(|_| Error::DatabaseError {
                        operation: "to_document",
                        with: "attachment",
                    })?,
                );
    
                remove_icon = true;
            }

            get_collection("channels")
            .update_one(
                doc! { "_id": &id },
                doc! { "$set": &set },
                None
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "channel" })?;

            ClientboundNotification::ChannelUpdate {
                id: id.clone(),
                data: json!(set),
            }
            .publish(id.clone())
            .await
            .ok();

            if let Some(name) = info.name {
                Message::create(
                    "00000000000000000000000000".to_string(),
                    id.clone(),
                    Content::SystemMessage(SystemMessage::ChannelRenamed {
                        name,
                        by: user.id,
                    }),
                )
                .publish(&target)
                .await
                .ok();
            }

            if remove_icon {
                if let Some(old_icon) = icon {
                    old_icon.delete().await?;
                }
            }

            Ok(())
        }
        _ => Err(Error::InvalidOperation),
    }
}
