use chrono::Utc;
use revolt_config::config;
use revolt_database::{
    util::{
        oauth2,
        reference::Reference,
    },
    Database,
};
use revolt_models::v0;
use revolt_result::{create_error, Result};
use rocket::{form::Form, serde::json::Json, State};
use redis_kiss::AsyncCommands;

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

    let claims = oauth2::decode_token(&config.api.security.token_secret, &info.code)
        .map_err(|_| create_error!(NotAuthenticated))?;

    if let Some((client_secret, bot_secret)) = info.client_secret.as_ref().zip(bot.oauth2.as_ref().and_then(|oauth2| oauth2.secret.as_ref())) {
        if client_secret != bot_secret {
            return Err(create_error!(NotAuthenticated))
        }
    } else if let Some((code_verifier, method)) = info.code_verifier.as_ref().zip(claims.code_challange_method) {
        let mut conn = redis_kiss::get_connection()
            .await
            .map_err(|_| create_error!(InternalError))?;

        let server_code_challenge = conn.get::<_, String>(format!("oauth2:{}:code_challenge", &info.code))
            .await
            .map_err(|_| create_error!(InternalError))?;

        let is_valid = match method {
            v0::OAuth2CodeChallangeMethod::Plain => &server_code_challenge == code_verifier,
            v0::OAuth2CodeChallangeMethod::S256 => pkce::code_challenge(code_verifier.as_bytes()) == server_code_challenge,
        };

        if !is_valid {
            return Err(create_error!(NotAuthenticated))
        }
    }

    if claims.client_id != info.client_id || claims.exp < Utc::now().timestamp() {
        return Err(create_error!(NotAuthenticated));
    };

    match info.grant_type {
        v0::OAuth2GrantType::AuthorizationCode => {
            if claims.r#type != oauth2::TokenType::Auth {
                return Err(create_error!(InvalidOperation))
            }

            let token = oauth2::encode_token(
                &config.api.security.token_secret,
                oauth2::TokenType::Access,
                claims.sub,
                claims.client_id,
                claims.redirect_uri,
                claims.scope.clone(),
                None,
            )
            .map_err(|_| create_error!(InternalError))?;

            Ok(Json(v0::OAuth2TokenExchangeResponse {
                access_token: token,
                token_type: "OAuth2".to_string(),
                scope: claims.scope,
            }))
        },
        v0::OAuth2GrantType::Implicit => {
            Err(create_error!(InvalidOperation))
        }
    }
}
