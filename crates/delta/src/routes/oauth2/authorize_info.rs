use revolt_database::{util::reference::Reference, Database, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Authorize OAuth Information
///
/// Fetches the information needed for displaying on the OAuth grant page
#[openapi(tag = "OAuth2")]
#[get("/authorize?<info..>")]
pub async fn info(
    db: &State<Database>,
    user: User,
    info: v0::OAuth2AuthorizationForm,
) -> Result<Json<v0::OAuth2AuthorizeInfoResponse>> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    };

    let bot = Reference::from_unchecked(&info.client_id).as_bot(db).await?;
    let bot_user = Reference::from_unchecked(&bot.id).as_user(db).await?;
    let public_bot = bot.clone().into_public_bot(bot_user);

    let Some(oauth2) = &bot.oauth2 else {
        return Err(create_error!(InvalidOperation));
    };

    if !oauth2.redirects.contains(&info.redirect_uri) || v0::OAuth2Scope::scopes_from_str(&info.scope).is_none() {
        return Err(create_error!(InvalidOperation));
    };

    Ok(Json(v0::OAuth2AuthorizeInfoResponse {
        bot: public_bot,
        user: user.into(db, None).await,
        allowed_scopes: oauth2.allowed_scopes.clone()
            .into_iter()
            .map(|(scope, value)| (scope.into(), value.into()))
            .collect()
    }))
}