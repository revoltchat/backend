use revolt_quark::{
    models::{server_member::RemovalIntention, User},
    Db, EmptyResponse, Ref, Result,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// # Query Parameters
#[derive(Validate, Serialize, Deserialize, JsonSchema, FromForm)]
pub struct OptionsServerDelete {
    /// Whether to not send a leave message
    leave_silently: Option<bool>,
}

/// # Delete / Leave Server
///
/// Deletes a server if owner otherwise leaves.
#[openapi(tag = "Server Information")]
#[delete("/<target>?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsServerDelete,
) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    let member = db.fetch_member(&target.id, &user.id).await?;

    if server.owner == user.id {
        server.delete(db).await
    } else {
        server
            .remove_member(
                db,
                member,
                RemovalIntention::Leave,
                options.leave_silently.unwrap_or_default(),
            )
            .await
    }
    .map(|_| EmptyResponse)
}
