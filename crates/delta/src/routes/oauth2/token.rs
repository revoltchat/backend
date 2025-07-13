use chrono::Utc;
use iso8601_timestamp::Timestamp;
use revolt_config::config;
use revolt_database::{
    util::{
        oauth2,
        reference::Reference,
    }, AuthorizedBot, AuthorizedBotId, Database
};
use revolt_models::v0;
use revolt_result::{create_error, ErrorType, Result};
use rocket::{form::Form, serde::json::Json, State};

/// # OAuth Token Exchange
///
///
#[openapi(tag = "OAuth2")]
#[post("/token", data = "<info>")]
pub async fn token(
    db: &State<Database>,
    info: Form<v0::OAuth2TokenExchangeForm>,
) -> Result<Json<v0::OAuth2TokenExchangeResponse>> {
    let bot = Reference::from_unchecked(info.client_id.clone())
        .as_bot(db)
        .await?;

    let config = config().await;

    let (token, token_type) = match (&info.code, &info.refresh_token) {
        (Some(code), None) => {
            (code, oauth2::TokenType::Auth)
        },
        (None, Some(refresh_token)) => {
            (refresh_token, oauth2::TokenType::Refresh)
        },
        (Some(_), Some(_)) | (None, None) => {
            return Err(create_error!(InvalidOperation))
        }
    };

    let claims = oauth2::decode_token(&config.api.security.token_secret, token)
        .map_err(|_| create_error!(NotAuthenticated))?;

    if claims.r#type != token_type || claims.client_id != info.client_id || claims.exp < Utc::now().timestamp() {
        return Err(create_error!(NotAuthenticated))
    };

    if let Ok(authorized_bot) = db.fetch_authorized_bot(&AuthorizedBotId { user: claims.sub.clone(), bot: claims.client_id.clone() }).await {
        if authorized_bot.deauthorized_at.is_some() {
            return Err(create_error!(NotAuthenticated));
        }
    };

    // TODO: track used tokens and dont allow them to be used twice
    //  - refresh token
    //  - auth tokens (only last 5 mins but still should be considered)

    match info.grant_type {
        v0::OAuth2GrantType::AuthorizationCode => {
            if claims.r#type != oauth2::TokenType::Auth {
                return Err(create_error!(InvalidOperation))
            }

            if let Some((client_secret, bot_secret)) = info.client_secret.as_ref().zip(bot.oauth2.as_ref().and_then(|oauth2| oauth2.secret.as_ref())) {
                if client_secret != bot_secret {
                    return Err(create_error!(NotAuthenticated))
                }
            } else if let Some((code_verifier, method)) = info.code_verifier.as_ref().zip(claims.code_challange_method) {
                let Some(server_code_challenge) = oauth2::get_code_challange(token).await? else {
                    return Err(create_error!(NotAuthenticated))
                };

                let is_valid = match method {
                    v0::OAuth2CodeChallangeMethod::Plain => &server_code_challenge == code_verifier,
                    v0::OAuth2CodeChallangeMethod::S256 => pkce::code_challenge(code_verifier.as_bytes()) == server_code_challenge,
                };

                if !is_valid {
                    return Err(create_error!(NotAuthenticated))
                }
            } else {
                return Err(create_error!(NotAuthenticated))
            }

            let token = oauth2::encode_token(
                &config.api.security.token_secret,
                oauth2::TokenType::Access,
                claims.sub.clone(),
                claims.client_id.clone(),
                claims.redirect_uri.clone(),
                claims.scopes.clone(),
                None,
            )
            .map_err(|_| create_error!(InternalError))?;

            let refresh_token = oauth2::encode_token(
                &config.api.security.token_secret,
                oauth2::TokenType::Refresh,
                claims.sub.clone(),
                claims.client_id.clone(),
                claims.redirect_uri.clone(),
                claims.scopes.clone(),
                None,
            )
            .map_err(|_| create_error!(InternalError))?;

            let authorized_bot_id = AuthorizedBotId { bot: claims.client_id.clone(), user: claims.sub.clone() };
            let auth_bot = db.fetch_authorized_bot(&authorized_bot_id).await;
            println!("{auth_bot:?}");

            if auth_bot.is_err_and(|err| err.error_type == ErrorType::NotFound) {
                println!("inserting");

                db.insert_authorized_bot(&AuthorizedBot {
                    id: authorized_bot_id,
                    created_at: Timestamp::now_utc(),
                    deauthorized_at: None,
                    scope: claims.scopes.iter().map(|&scope| scope.into()).collect()
                }).await?;
            }

            Ok(Json(v0::OAuth2TokenExchangeResponse {
                access_token: token,
                refresh_token: Some(refresh_token),
                token_type: "OAuth2".to_string(),
                scope: claims.scopes.iter().map(|scope| scope.to_string()).collect::<Vec<_>>().join(" "),
            }))
        },
        v0::OAuth2GrantType::RefreshToken => {
            if claims.r#type != oauth2::TokenType::Refresh {
                return Err(create_error!(InvalidOperation))
            };

            let token = oauth2::encode_token(
                &config.api.security.token_secret,
                oauth2::TokenType::Access,
                claims.sub.clone(),
                claims.client_id.clone(),
                claims.redirect_uri.clone(),
                claims.scopes.clone(),
                None,
            )
            .map_err(|_| create_error!(InternalError))?;

            let refresh_token = oauth2::encode_token(
                &config.api.security.token_secret,
                oauth2::TokenType::Refresh,
                claims.sub.clone(),
                claims.client_id.clone(),
                claims.redirect_uri.clone(),
                claims.scopes.clone(),
                None,
            )
            .map_err(|_| create_error!(InternalError))?;

            Ok(Json(v0::OAuth2TokenExchangeResponse {
                access_token: token,
                refresh_token: Some(refresh_token),
                token_type: "OAuth2".to_string(),
                scope: claims.scopes.iter().map(|scope| scope.to_string()).collect::<Vec<_>>().join(" "),
            }))
        }
        v0::OAuth2GrantType::Implicit => {
            // token is already an access token so this endpoint does not need to be called - in theory this should be unreachable
            Err(create_error!(InvalidOperation))
        }
    }
}
