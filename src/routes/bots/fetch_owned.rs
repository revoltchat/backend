use revolt_quark::{
    models::{Bot, User},
    Db, Error, Result,
};
use rocket::serde::json::Json;
use serde::Serialize;

/// # Owned Bots Response
///
/// Both lists are sorted by their IDs.
#[derive(Serialize, JsonSchema)]
pub struct OwnedBotsResponse {
    /// Bot objects
    bots: Vec<Bot>,
    /// User objects
    users: Vec<User>,
}

/// # Fetch Owned Bots
///
/// Fetch all of the bots that you have control over.
#[openapi(tag = "Bots")]
#[get("/@me")]
pub async fn fetch_owned_bots(db: &Db, user: User) -> Result<Json<OwnedBotsResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let mut bots = db.fetch_bots_by_user(&user.id).await?;
    let user_ids = bots
        .iter()
        .map(|x| x.id.to_owned())
        .collect::<Vec<String>>();

    let mut users = db.fetch_users(&user_ids).await?;

    // Ensure the lists match up exactly.
    bots.sort_by(|a, b| a.id.cmp(&b.id));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(Json(OwnedBotsResponse { users, bots }))
}
