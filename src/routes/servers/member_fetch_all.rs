use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::bson::{doc, from_document, Document};
use rocket_contrib::json::JsonValue;

// ! FIXME: this is a temporary route while permissions are being worked on.

#[get("/<target>/members")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let target = target.fetch_server().await?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_server(&target)
        .for_server()
        .await?;

    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let members = get_collection("server_members")
        .find(
            doc! {
                "_id.server": target.id
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "server_members",
        })?
        .filter_map(async move |s| s.ok())
        .collect::<Vec<Document>>()
        .await
        .into_iter()
        .filter_map(|x| from_document(x).ok())
        .collect::<Vec<Member>>();

    let member_ids = members
        .iter()
        .map(|m| m.id.user.clone())
        .collect::<Vec<String>>();

    Ok(json!({
        "members": members,
        "users": user.fetch_multiple_users(member_ids).await?
    }))
}
