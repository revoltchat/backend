use revolt_quark::{
    models::{server_member::RemovalIntention, User},
    Db, EmptyResponse, Ref, Result,
};

/// # Delete / Leave Server
///
/// Deletes a server if owner otherwise leaves.
#[openapi(tag = "Server Information")]
#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    let member = db.fetch_member(&target.id, &user.id).await?;

    if server.owner == user.id {
        server.delete(db).await
    } else {
        server
            .remove_member(db, member, RemovalIntention::Leave)
            .await
    }
    .map(|_| EmptyResponse)
}
