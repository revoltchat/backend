use revolt_config::config;
use revolt_database::{
    util::{oauth2, reference::Reference}, Database, User
};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use rocket::{form::Form, serde::json::Json, State};
use redis_kiss::AsyncCommands;

/// # OAuth2 Authorization Auth
///
/// Generates an OAuth2 code to be passed to the redirect URI.
#[openapi(tag = "OAuth2")]
#[post("/authorize?<info..>")]
pub async fn auth(
    db: &State<Database>,
    user: User,
    info: v0::OAuth2AuthorizationForm,
) -> Result<Json<v0::OAuth2AuthorizeAuthResponse>> {
    if user.bot.is_some() {
        return Err(create_error!(IsBot));
    };

    let bot = Reference::from_unchecked(info.client_id.clone())
        .as_bot(db)
        .await?;

    let Some(oauth2) = &bot.oauth2 else {
        return Err(create_error!(InvalidOperation));
    };

    let Some(scopes) = v0::OAuth2Scope::scopes_from_str(&info.scope) else {
        return Err(create_error!(InvalidOperation));
    };

    if scopes.into_iter().any(|scope| !oauth2.allowed_scopes.contains_key(&scope.into()))
        || !oauth2.redirects.contains(&info.redirect_uri)
        || v0::OAuth2Scope::scopes_from_str(&info.scope).is_none()
    {
        return Err(create_error!(InvalidOperation));
    };

    let config = config().await;

    let token = match info.response_type {
        // implicit
        v0::OAuth2ResponseType::Code => {
            if info.state.is_some() || info.code_challenge.is_some() || info.code_challenge_method.is_some() {
                return Err(create_error!(InvalidOperation));
            };

            oauth2::encode_token(
                &config.api.security.token_secret,
                oauth2::TokenType::Auth,
                user.id.clone(),
                info.client_id.clone(),
                info.redirect_uri.clone(),
                info.scope.clone(),
                info.code_challenge_method,
            )
            .map_err(|_| create_error!(InternalError))?
        },
        // authorization code
        v0::OAuth2ResponseType::Token => {
            if let Some(((state, code_challange), code_challange_method)) = info.state.as_ref().zip(info.code_challenge.as_ref()).zip(info.code_challenge_method) {
                let is_valid = match code_challange_method {
                    v0::OAuth2CodeChallangeMethod::Plain => state == code_challange,
                    v0::OAuth2CodeChallangeMethod::S256 => &pkce::code_challenge(state.as_bytes()) == code_challange,
                };

                if !is_valid || state.len() <= 43 || state.len() >= 128 {
                    return Err(create_error!(InvalidOperation))
                }

            } else if !oauth2.public {
                return Err(create_error!(InvalidOperation))
            };

            let token = oauth2::encode_token(
                &config.api.security.token_secret,
                oauth2::TokenType::Access,
                user.id.clone(),
                info.client_id.clone(),
                info.redirect_uri.clone(),
                info.scope.clone(),
                info.code_challenge_method,
            )
            .map_err(|_| create_error!(InternalError))?;

            if let Some(code_challenge) = info.code_challenge.as_ref() {
                oauth2::add_code_challange(&token, code_challenge).await?;
            };

            token

        },
    };

    let redirect_uri = format!("{}/?code={token}", &info.redirect_uri);

    Ok(Json(v0::OAuth2AuthorizeAuthResponse { redirect_uri }))
}
