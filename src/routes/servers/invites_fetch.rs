use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use rocket::serde::json::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInvite {
    #[serde(rename = "_id")]
    code: String,
    creator: String,
    channel: String,
}

#[get("/<target>/invites")]
pub async fn req(user: User, target: Ref) -> Result<Value> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_manage_server() {
        Err(Error::MissingPermission)?
    }

    let mut cursor = get_collection("channel_invites")
        .find(
            doc! {
                "server": target.id
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "channel_invites",
        })?;

    let mut invites = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            if let Ok(invite) = from_document::<Invite>(doc) {
                invites.push(invite);
            }
        }
    }

    Ok(json!(invites))
}
