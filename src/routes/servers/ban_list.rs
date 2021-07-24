use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::options::FindOptions;
use serde::{Serialize, Deserialize};
use rocket_contrib::json::JsonValue;
use mongodb::bson::{doc, from_document};

#[derive(Serialize, Deserialize)]
struct BannedUser {
    _id: String,
    username: String,
    avatar: Option<File>
}

#[get("/<target>/bans")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_ban_members() {
        return Err(Error::MissingPermission);
    }

    let mut cursor = get_collection("server_bans")
        .find(
            doc! {
                "_id.server": target.id
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "server_bans",
        })?;

    let mut bans = vec![];
    let mut user_ids = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            if let Ok(ban) = from_document::<Ban>(doc) {
                user_ids.push(ban.id.user.clone());
                bans.push(ban);
            }
        }
    }

    let mut cursor = get_collection("users")
        .find(
            doc! {
                "_id": {
                    "$in": user_ids
                }
            },
            FindOptions::builder()
                .projection(doc! {
                    "username": 1,
                    "avatar": 1
                })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "users",
        })?;

    let mut users = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            if let Ok(user) = from_document::<BannedUser>(doc) {
                users.push(user);
            }
        }
    }

    Ok(json!({
        "users": users,
        "bans": bans
    }))
}
