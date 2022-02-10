use revolt_quark::{models::User, perms, Db, Error, Ref, Result};

use rocket::serde::json::Value;

#[get("/<target>/members")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Value> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc_server(db)
        .await
        .get_view()
    {
        return Err(Error::NotFound);
    }

    let members = db.fetch_all_members(&server.id).await?;

    let mut user_ids = vec![];
    for member in &members {
        user_ids.push(member.id.user.clone());
    }

    let users = db.fetch_users(&user_ids).await?;

    Ok(json!({
        "members": members,
        "users": users
    }))
}
