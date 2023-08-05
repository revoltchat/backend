use futures::future::join_all;
use revolt_database::{Database, User};
use revolt_models::v0::OwnedBotsResponse;
use revolt_result::Result;
use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Owned Bots
///
/// Fetch all of the bots that you have control over.
#[openapi(tag = "Bots")]
#[get("/@me")]
pub async fn fetch_owned_bots(db: &State<Database>, user: User) -> Result<Json<OwnedBotsResponse>> {
    let mut bots = db.fetch_bots_by_user(&user.id).await?;
    let user_ids = bots
        .iter()
        .map(|x| x.id.to_owned())
        .collect::<Vec<String>>();

    let mut users = db.fetch_users(&user_ids).await?;

    // Ensure the lists match up exactly.
    bots.sort_by(|a, b| a.id.cmp(&b.id));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(Json(OwnedBotsResponse {
        users: join_all(users.into_iter().map(|user| user.into_self())).await,
        bots: bots.into_iter().map(|bot| bot.into()).collect(),
    }))
}
