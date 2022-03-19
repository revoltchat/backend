use revolt_quark::{models::User, perms, presence::presence_filter_online, Db, Ref, Result};

use rocket::serde::json::Value;

/// # Fetch Members
///
/// Fetch all server members.
#[openapi(tag = "Server Members")]
#[get("/<target>/members")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Value> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    let mut members = db.fetch_all_members(&server.id).await?;

    let mut user_ids = vec![];
    for member in &members {
        user_ids.push(member.id.user.clone());
    }

    let online_ids = presence_filter_online(&user_ids).await;
    let mut users = db
        .fetch_users(&user_ids)
        .await?
        .into_iter()
        .map(|mut user| {
            user.online = Some(online_ids.contains(&user.id));
            user.foreign()
        })
        .collect::<Vec<User>>();

    // Ensure the lists match up exactly.
    members.sort_by(|a, b| a.id.user.cmp(&b.id.user));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(json!({
        "members": members,
        "users": users
    }))
}
