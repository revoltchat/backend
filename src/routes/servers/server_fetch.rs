use crate::database::*;
use crate::util::result::{Error, Result};
use mongodb::bson::{doc, from_document};
use futures::StreamExt;

use rocket::serde::json::Value;

#[get("/<target>")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let mut target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server().await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let mut channel_cursor = get_collection("channels")
        .find(
            doc! {
                "_id": {
                    "$in": &target.channels
                }
            },
            None
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "channels",
        })?;

    // Remove channels that the user can't view
    while let Some(maybe_document) = channel_cursor.next().await {
        if let Ok(document) = maybe_document {
            let channel: Channel = from_document(document)
                .map_err(|_| Error::DatabaseError {
                    operation: "from_document",
                    with: "channel",
                })?;

            let channel_perms =
                permissions::PermissionCalculator::new(&user)
                    .with_channel(&channel)
                    .for_channel().await?;

            if !channel_perms.get_view() {
                let maybe_index = target.channels.iter().position(|id| *id == channel.id());

                if let Some(index) = maybe_index {
                    target.channels.remove(index);
                }
            }
        }
    }

    Ok(json!(target))
}
