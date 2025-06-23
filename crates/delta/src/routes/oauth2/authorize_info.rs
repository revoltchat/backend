use revolt_database::{util::reference::Reference, Database, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use rocket::{form::Form, serde::json::Json, State};

/// # Authorize OAuth Information
///
/// Fetches the information needed for displaying on the OAuth grant page
#[openapi(tag = "OAuth2")]
#[get("/authorize", data="<info>")]
pub async fn info(
    db: &State<Database>,
    user: User,
    info: Form<v0::OAuth2AuthorizationForm>,
) -> Result<Json<v0::OAuth2AuthorizeInfoResponse>> {
    let bot = Reference::from_unchecked(info.client_id.to_string()).as_bot(db).await?;

    let Some(oauth2) = &bot.oauth2 else {
        return Err(create_error!(InvalidOperation));
    };

    if !oauth2.redirects.contains(&info.redirect_uri) || v0::OAuth2Scope::scopes_from_str(&info.scope).is_none() {
        return Err(create_error!(InvalidOperation));
    };

    Ok(Json(v0::OAuth2AuthorizeInfoResponse {
        bot: bot.into(),
        user: user.into(db, None).await
    }))
}