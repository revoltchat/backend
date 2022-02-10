use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result};

#[put("/<target>/ack")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_view()
    {
        return Err(Error::NotFound);
    }

    db.acknowledge_channels(&user.id, &server.channels)
        .await
        .map(|_| EmptyResponse)
}
