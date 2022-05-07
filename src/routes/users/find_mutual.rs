use revolt_quark::models::User;
use revolt_quark::{perms, Database, Error, Ref, Result};

use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

/// # Mutual Friends and Servers Response
#[derive(Serialize, JsonSchema)]
pub struct MutualResponse {
    /// Array of mutual user IDs that both users are friends with
    users: Vec<String>,
    /// Array of mutual server IDs that both users are in
    servers: Vec<String>,
}

/// # Fetch Mutual Friends And Servers
///
/// Retrieve a list of mutual friends and servers with another user.
#[openapi(tag = "Relationships")]
#[get("/<target>/mutual")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<MutualResponse>> {
    if target.id == user.id {
        return Err(Error::InvalidOperation);
    }

    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let target = target.as_user(db).await?;

    if perms(&user)
        .user(&target)
        .calc_user(db)
        .await
        .get_view_profile()
    {
        Ok(Json(MutualResponse {
            users: db.fetch_mutual_user_ids(&user.id, &target.id).await?,
            servers: db.fetch_mutual_server_ids(&user.id, &target.id).await?,
        }))
    } else {
        Err(Error::NotFound)
    }
}
