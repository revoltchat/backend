use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result, ServerPermission};

#[delete("/<target>/members/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_kick_members()
    {
        return Err(Error::MissingPermission {
            permission: ServerPermission::KickMembers as i32,
        });
    }

    let member = member.as_member(db, &server.id).await?;
    // ! FIXME: calculate permission against member

    member.delete(db).await.map(|_| EmptyResponse)
}
