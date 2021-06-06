use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use rocket_contrib::json::JsonValue;
use mongodb::bson::{doc, from_document};

#[get("/<target>/bans")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;
    
    if !perm.get_manage_members() {
        Err(Error::MissingPermission)?
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
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            if let Ok(ban) = from_document::<Ban>(doc) {
                bans.push(ban);
            }
        }
    }
    
    Ok(json!(bans))
}
