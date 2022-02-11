use revolt_quark::{models::User, Db, EmptyResponse, Ref, Result};

#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    let member = db.fetch_member(&target.id, &user.id).await?;

    if server.owner == user.id {
        db.delete_server(&server.id).await?;
    } else {
        db.delete_member(&member.id).await?;
    }

    Ok(EmptyResponse)
}
