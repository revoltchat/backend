use revolt_quark::{
    models::{Member, User},
    perms, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

/// # Query Parameters
#[derive(Deserialize, JsonSchema, FromForm)]
pub struct OptionsQueryMembers {
    /// String to search for
    query: String,

    /// Discourage use of this API
    experimental_api: bool,
}

/// # Query members by name
#[derive(Serialize, JsonSchema)]
pub struct MemberQueryResponse {
    /// List of members
    members: Vec<Member>,
    /// List of users
    users: Vec<User>,
}

/// # Query members by name
///
/// Query members by a given name, this API is not stable and will be removed in the future.
#[openapi(tag = "Server Members")]
#[get("/<target>/members_experimental_query?<options..>")]
pub async fn member_experimental_query(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsQueryMembers,
) -> Result<Json<MemberQueryResponse>> {
    if !options.experimental_api {
        return Err(Error::InternalError);
    }

    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    let mut members = db.fetch_all_members(&server.id).await?;

    let mut user_ids = vec![];
    for member in &members {
        user_ids.push(member.id.user.clone());
    }

    let mut users = User::fetch_foreign_users(db, &user_ids).await?;

    // Ensure the lists match up exactly
    members.sort_by(|a, b| a.id.user.cmp(&b.id.user));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    // Filter all matches
    let mut zipped_vec: Vec<(Member, User)> = members
        .into_iter()
        .zip(users)
        .filter(|(member, user)| {
            if let Some(nickname) = &member.nickname {
                nickname.contains(&options.query)
            } else {
                user.username.contains(&options.query)
            }
        })
        .collect();

    // Sort remaining matches by length
    zipped_vec.sort_by(|(member_a, user_a), (member_b, user_b)| {
        let left = member_a.nickname.as_ref().unwrap_or(&user_a.username);
        let right = member_b.nickname.as_ref().unwrap_or(&user_b.username);
        left.len().cmp(&right.len())
    });

    // Take the first five and return them
    let (members, users) = zipped_vec.into_iter().take(10).unzip();
    Ok(Json(MemberQueryResponse { members, users }))
}
