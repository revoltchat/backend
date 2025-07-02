use revolt_models::v0;
use rocket::{serde::json::Json, State};
use revolt_database::{Database, User};
use revolt_result::Result;


#[openapi(tag = "OAuth2")]
#[get("/authorized_bots")]
pub async fn authorized_bots(
    db: &State<Database>,
    user: User
) -> Result<Json<Vec<v0::AuthorizedBot>>> {
    let authorized_bots = db.fetch_users_authorized_bots(&user.id).await?;

    Ok(Json(authorized_bots
        .into_iter()
        .map(|bot| bot.into())
        .collect()
    ))
}