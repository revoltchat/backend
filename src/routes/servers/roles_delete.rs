use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result};

#[delete("/<target>/roles/<role_id>")]
pub async fn req(db: &Db, user: User, target: Ref, role_id: String) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_manage_roles()
    {
        return Err(Error::NotFound);
    }

    db.delete_role(&server.id, &role_id)
        .await
        .map(|_| EmptyResponse)
}
