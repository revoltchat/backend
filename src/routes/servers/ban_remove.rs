use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result, ServerPermission};

#[delete("/<server>/bans/<target>")]
pub async fn req(db: &Db, user: User, server: Ref, target: Ref) -> Result<EmptyResponse> {
    let server = server.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_ban_members()
    {
        return Err(Error::MissingPermission {
            permission: ServerPermission::BanMembers as i32,
        });
    }

    let ban = target.as_ban(db, &server.id).await?;
    db.delete_ban(&ban.id).await.map(|_| EmptyResponse)
}
