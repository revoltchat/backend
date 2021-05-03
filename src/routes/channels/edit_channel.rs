use crate::{database::*, notifications::events::RemoveChannelField};
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
    remove: Option<RemoveChannelField>
}

#[patch("/<target>", data = "<data>")]
pub async fn req(user: User, target: Ref, data: Json<Data>) -> Result<()> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

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
            let mut unset = doc! {};

            let mut remove_icon = false;
            if let Some(remove) = &data.remove {
                match remove {
                    RemoveChannelField::Icon => {
                        unset.insert("icon", 1);
                        remove_icon = true;
                    }
                }
            }

            if let Some(name) = &data.name {
                set.insert("name", name);
            }

            if let Some(description) = &data.description {
                set.insert("description", description);
            }

            if let Some(attachment_id) = &data.icon {
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

            let mut operations = doc! {};
            if set.len() > 0 {
                operations.insert("$set", &set);
            }
        
            if unset.len() > 0 {
                operations.insert("$unset", unset);
            }

            get_collection("channels")
            .update_one(
                doc! { "_id": &id },
                operations,
                None
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "channel" })?;

            ClientboundNotification::ChannelUpdate {
                id: id.clone(),
                data: json!(set),
                clear: data.remove
            }
            .publish(id.clone())
            .await
            .ok();

            if let Some(name) = data.name {
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
