use revolt_quark::{
    models::{
        server::{Category, FieldsServer, PartialServer, SystemMessageChannels},
        File, Server, User,
    },
    perms, Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
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

    #[validate]
    categories: Option<Vec<Category>>,
    system_messages: Option<SystemMessageChannels>,

    nsfw: Option<bool>,
    analytics: Option<bool>,

    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsServer>>,
}

#[patch("/<target>", data = "<data>")]
pub async fn req(db: &Db, user: User, target: Ref, data: Json<Data>) -> Result<Json<Server>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut server = target.as_server(db).await?;
    let permissions = perms(&user).server(&server).calc(db).await?;
    if !permissions.can_view_channel() {
        return Err(Error::NotFound);
    }

    // Check permissions
    if data.name.is_none()
        && data.description.is_none()
        && data.icon.is_none()
        && data.banner.is_none()
        && data.system_messages.is_none()
        && data.categories.is_none()
        && data.nsfw.is_none()
        && data.analytics.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(server));
    } else if (data.name.is_some()
        || data.description.is_some()
        || data.icon.is_some()
        || data.banner.is_some()
        || data.system_messages.is_some()
        || data.nsfw.is_some()
        || data.analytics.is_some()
        || data.remove.is_some())
        && !permissions.can_manage_server()
    {
        return Error::from_permission(Permission::ManageServer);
    } else if (data.categories.is_some()) && !permissions.can_manage_channel() {
        return Error::from_permission(Permission::ManageChannel);
    }

    let Data {
        name,
        description,
        icon,
        banner,
        categories,
        system_messages,
        nsfw,
        analytics,
        remove,
    } = data;

    let mut partial = PartialServer {
        name,
        description,
        categories,
        system_messages,
        nsfw,
        analytics,
        ..Default::default()
    };

    // 1. Remove fields from object
    if let Some(fields) = &remove {
        if fields.contains(&FieldsServer::Banner) {
            if let Some(banner) = &server.banner {
                db.mark_attachment_as_deleted(&banner.id).await?;
            }
        }

        if fields.contains(&FieldsServer::Icon) {
            if let Some(icon) = &server.icon {
                db.mark_attachment_as_deleted(&icon.id).await?;
            }
        }
    }

    // 2. Apply new icon
    if let Some(icon) = icon {
        partial.icon = Some(File::use_server_icon(db, &icon, &server.id).await?);
        server.icon = partial.icon.clone();
    }

    // 3. Apply new banner
    if let Some(banner) = banner {
        partial.banner = Some(File::use_server_icon(db, &banner, &server.id).await?);
        server.banner = partial.banner.clone();
    }

    // 4. Validate changes
    if let Some(system_messages) = &partial.system_messages {
        let channels = system_messages.clone().into_channel_ids();
        if !db
            .check_channels_exist(&channels.into_iter().collect())
            .await?
        {
            return Err(Error::NotFound);
        }
    }

    server
        .update(db, partial, remove.unwrap_or_default())
        .await?;

    Ok(Json(server))
}
