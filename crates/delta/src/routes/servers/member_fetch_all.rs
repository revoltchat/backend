use revolt_quark::{
    models::{Member, User},
    perms, Db, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

/// # Query Parameters
#[derive(Deserialize, JsonSchema, FromForm)]
pub struct OptionsFetchAllMembers {
    /// Whether to exclude offline users
    exclude_offline: Option<bool>,
}

/// # Member List
///
/// Both lists are sorted by ID.
#[derive(Serialize, JsonSchema)]
pub struct AllMemberResponse {
    /// List of members
    members: Vec<Member>,
    /// List of users
    users: Vec<User>,
}

/// # Fetch Members
///
/// Fetch all server members.
#[openapi(tag = "Server Members")]
#[get("/<target>/members?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsFetchAllMembers,
) -> Result<Json<AllMemberResponse>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    let mut members = db.fetch_all_members(&server.id).await?;

    let mut user_ids = vec![];
    for member in &members {
        user_ids.push(member.id.user.clone());
    }

    let mut users = User::fetch_foreign_users(db, &user_ids).await?;

    // Ensure the lists match up exactly.
    members.sort_by(|a, b| a.id.user.cmp(&b.id.user));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    // Optionally, remove all offline user entries.
    if let Some(true) = options.exclude_offline {
        let mut iter = users.iter();
        members.retain(|_| iter.next().unwrap().online.unwrap_or(false));
        users.retain(|user| user.online.unwrap_or(false));
    }

    Ok(Json(AllMemberResponse { members, users }))
}
