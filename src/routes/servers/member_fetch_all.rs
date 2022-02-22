use revolt_quark::{models::User, perms, presence::presence_filter_online, Db, Error, Ref, Result};

use rocket::serde::json::Value;

#[get("/<target>/members")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Value> {
    let server = target.as_server(db).await?;
    if !perms(&user)
        .server(&server)
        .calc(db)
        .await
        .can_view_channel()
    {
        return Err(Error::NotFound);
    }

    let members = db.fetch_all_members(&server.id).await?;

    let mut user_ids = vec![];
    for member in &members {
        user_ids.push(member.id.user.clone());
    }

    let online_ids = presence_filter_online(&user_ids).await;
    let users = db
        .fetch_users(&user_ids)
        .await?
        .into_iter()
        .map(|mut user| {
            user.online = Some(online_ids.contains(&user.id));
            user.foreign()
        })
        .collect::<Vec<User>>();

    Ok(json!({
        "members": members,
        "users": users
    }))
}
