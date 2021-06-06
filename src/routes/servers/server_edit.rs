use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};
use crate::{database::*, notifications::events::RemoveServerField};

use mongodb::bson::{doc, to_document};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    remove: Option<RemoveServerField>,
}

#[patch("/<target>", data = "<data>")]
pub async fn req(user: User, target: Ref, data: Json<Data>) -> Result<()> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if data.name.is_none() && data.icon.is_none() && data.banner.is_none() && data.remove.is_none()
    {
        return Ok(());
    }

    let target = target.fetch_server().await?;
    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_server() {
        Err(Error::MissingPermission)?
    }

    let mut set = doc! {};
    let mut unset = doc! {};

    let mut remove_icon = false;
    let mut remove_banner = false;
    if let Some(remove) = &data.remove {
        match remove {
            RemoveServerField::Icon => {
                unset.insert("icon", 1);
                remove_icon = true;
            }
            RemoveServerField::Banner => {
                unset.insert("banner", 1);
                remove_banner = true;
            }
            RemoveServerField::Description => {
                unset.insert("description", 1);
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
        let attachment = File::find_and_use(&attachment_id, "icons", "object", &target.id).await?;
        set.insert(
            "icon",
            to_document(&attachment).map_err(|_| Error::DatabaseError {
                operation: "to_document",
                with: "attachment",
            })?,
        );

        remove_icon = true;
    }

    if let Some(attachment_id) = &data.banner {
        let attachment =
            File::find_and_use(&attachment_id, "banners", "server", &target.id).await?;
        set.insert(
            "banner",
            to_document(&attachment).map_err(|_| Error::DatabaseError {
                operation: "to_document",
                with: "attachment",
            })?,
        );

        remove_banner = true;
    }

    let mut operations = doc! {};
    if set.len() > 0 {
        operations.insert("$set", &set);
    }

    if unset.len() > 0 {
        operations.insert("$unset", unset);
    }

    if operations.len() > 0 {
        get_collection("servers")
            .update_one(doc! { "_id": &target.id }, operations, None)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "server",
            })?;
    }

    ClientboundNotification::ServerUpdate {
        id: target.id.clone(),
        data: json!(set),
        clear: data.remove,
    }
    .publish(target.id.clone());

    let Server { icon, banner, .. } = target;

    if remove_icon {
        if let Some(old_icon) = icon {
            old_icon.delete().await?;
        }
    }

    if remove_banner {
        if let Some(old_banner) = banner {
            old_banner.delete().await?;
        }
    }

    Ok(())
}
