use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Permission, Ref, Result};

#[delete("/<target>/roles/<role_id>")]
pub async fn req(db: &Db, user: User, target: Ref, role_id: String) -> Result<EmptyResponse> {
    let mut server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, Permission::ManageRole)
        .await?;

    // ! FIXME_PERMISSIONS

    if let Some(role) = server.roles.remove(&role_id) {
        role.delete(db, &server.id, &role_id)
            .await
            .map(|_| EmptyResponse)
    } else {
        Err(Error::NotFound)
    }
}
