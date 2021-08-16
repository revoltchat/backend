use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};

use crate::database::*;
use crate::database::permissions::channel::ChannelPermission;
use crate::database::permissions::server::ServerPermission;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result, EmptyResponse};

#[derive(Serialize, Deserialize)]
pub struct Values {
    server: u32,
    channel: u32
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    permissions: Values
}

#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(user: User, target: Ref, data: Json<Data>) -> Result<EmptyResponse> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_roles() {
        return Err(Error::MissingPermission);
    }

    let server_permissions: u32 = ServerPermission::View as u32 | data.permissions.server;
    let channel_permissions: u32 = ChannelPermission::View as u32 | data.permissions.channel;

    get_collection("servers")
        .update_one(
            doc! { "_id": &target.id },
            doc! {
                "$set": {
                    "default_permissions": [
                        server_permissions as i32,
                        channel_permissions as i32
                    ]
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "server"
        })?;

    ClientboundNotification::ServerUpdate {
        id: target.id.clone(),
        data: json!({
            "default_permissions": [
                server_permissions as i32,
                channel_permissions as i32
            ]
        }),
        clear: None
    }
    .publish(target.id);

    Ok(EmptyResponse {})
}
