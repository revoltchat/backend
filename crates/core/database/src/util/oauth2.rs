use chrono::{TimeDelta, Utc};
use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};
use revolt_models::v0;
use redis_kiss::AsyncCommands;
use revolt_result::Result;

#[cfg(feature = "rocket")]
use rocket::Request;

pub use jsonwebtoken::errors::{Error as JWTError, ErrorKind as JWTErrorKind};
use ulid::Ulid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType {
    Auth,
    Access,
    Refresh
}

impl TokenType {
    pub fn lifetime(self) -> TimeDelta {
        match self {
            TokenType::Access => TimeDelta::weeks(1),
            TokenType::Auth => TimeDelta::minutes(5),
            TokenType::Refresh => TimeDelta::weeks(4),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub jti: String,
    pub sub: String,
    pub exp: i64,

    pub r#type: TokenType,
    pub client_id: String,
    pub scopes: Vec<v0::OAuth2Scope>,
    pub redirect_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challange_method: Option<v0::OAuth2CodeChallangeMethod>,
}

#[allow(clippy::too_many_arguments)]
pub fn encode_token(
    token_secret: &str,
    token_type: TokenType,
    user_id: String,
    client_id: String,
    redirect_uri: String,
    scopes: Vec<v0::OAuth2Scope>,
    code_challange_method: Option<v0::OAuth2CodeChallangeMethod>,
) -> Result<String, Error> {
    let now = Utc::now();

    let exp = now
        .checked_add_signed(token_type.lifetime())
        .unwrap();

    println!("{now:?} {exp:}");

    let claims = Claims {
        jti: Ulid::new().to_string(),
        sub: user_id,
        exp: exp.timestamp(),

        r#type: token_type,
        client_id,
        scopes,
        redirect_uri,
        code_challange_method
    };

    let encoding_key = EncodingKey::from_secret(token_secret.as_bytes());

    encode(&Header::default(), &claims, &encoding_key)
}

pub fn decode_token(token_secret: &str, code: &str) -> Result<Claims, Error> {
    let decoding_key = DecodingKey::from_secret(token_secret.as_bytes());

    let data = decode(code, &decoding_key, &Validation::new(jsonwebtoken::Algorithm::HS256))?;

    Ok(data.claims)
}

#[cfg(feature = "rocket")]
pub fn scope_can_access_route(scope: v0::OAuth2Scope, request: &Request<'_>) -> bool {
    println!("{:?}", request.route());

    // TODO: figure out why request.segments(0..) is skipping the first segment
    let mut segments = request.uri().path().segments();

    if segments.get(0) == Some("0.8") {
        segments.next();  // skip first segment
    };

    // Extract the function name of the route
    let Some(route_name) = request
        .route()
        .and_then(|route| route.name.as_ref())
        .map(|name| name.as_ref())
    else {
        return false
    };

    match scope {
        v0::OAuth2Scope::ReadIdentify => route_name == "fetch_self",
        v0::OAuth2Scope::ReadServers => route_name == "fetch_self_servers" || route_name == "fetch_server",
        // This only grants access to the events websocket and not any routes
        v0::OAuth2Scope::Events => false,
        // TODO: maybe disallow revoking other sessions
        v0::OAuth2Scope::Full => true,
    }
}

#[cfg(feature = "rocket")]
pub fn scopes_can_access_route(scopes: &[v0::OAuth2Scope], request: &Request<'_>) -> bool {
    scopes
        .iter()
        .any(|&scope| scope_can_access_route(scope, request))
}

pub async fn add_code_challange(token: &str, code_challenge: &str) -> Result<()> {
    let mut conn = redis_kiss::get_connection()
        .await
        .map_err(|_| create_error!(InternalError))?;

    conn.pset_ex::<_, _, ()>(
        format!("oauth2:{token}:code_challenge"),
        code_challenge,
        TokenType::Access.lifetime().num_milliseconds() as usize
    )
    .await
    .map_err(|_| create_error!(InternalError))?;

    Ok(())
}

pub async fn get_code_challange(token: &str) -> Result<Option<String>> {
    let mut conn = redis_kiss::get_connection()
        .await
        .map_err(|_| create_error!(InternalError))?;

    conn.get(format!("oauth2:{token}:code_challenge"))
        .await
        .map_err(|_| create_error!(InternalError))
}