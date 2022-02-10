use revolt_quark::{models::User, perms, Db, EmptyResponse, Error, Ref, Result};

use mongodb::bson::doc;

#[delete("/<target>")]
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

    if server.owner == user.id {
        db.delete_server(&server.id).await?;
    } else {
        db.delete_member(&server.id, &user.id).await?;
    }

    Ok(EmptyResponse)
}
