use chrono::{TimeDelta, Utc};
use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};
use revolt_models::v0;

#[cfg(feature = "rocket")]
use rocket::{http::Method, Request};

// use redis_kiss::{get_connection, redis::Pipeline, AsyncCommands};
// use revolt_result::Result;

// pub async fn add_access_token(code: &str, redirect_uri: &str, client_id: &str) -> Result<()> {
//     let conn = get_connection()
//         .await
//         .map_err(|_| create_error!(InternalError))?;

//     // 10 mins
//     Pipeline::new()
//         .pset_ex(format!("oauth:{code}:client_id"), client_id, 600000)
//         .pset_ex(format!("oauth:{code}:redirect_uri"), redirect_uri, 600000)
//         .query_async::<_, ()>(&mut conn.into_inner())
//         .await
//         .map_err(|_| create_error!(InternalError))?;

//     Ok(())
// }

// pub struct CodeInfo {
//     pub redirect_uri: String,
//     pub client_id: String
// }

// pub async fn get_access_token_info(code: &str) -> Result<CodeInfo> {
//     let conn = get_connection()
//         .await
//         .map_err(|_| create_error!(InternalError))?;

//     let (redirect_uri, client_id) = Pipeline::new()
//         .get(format!("oauth:{code}:client_id"))
//         .get(format!("oauth:{code}:redirect_uri"))
//         .query_async::<_, (String, String)>(&mut conn.into_inner())
//         .await
//         .map_err(|_| create_error!(InternalError))?;

//     Ok(CodeInfo { redirect_uri, client_id })
// }

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
    pub sub: String,
    pub exp: i64,

    pub r#type: TokenType,
    pub client_id: String,
    pub scope: String,
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
    scope: String,
    code_challange_method: Option<v0::OAuth2CodeChallangeMethod>,
) -> Result<String, Error> {
    let exp = Utc::now()
        .checked_add_signed(token_type.lifetime())
        .unwrap()
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp,

        r#type: token_type,
        client_id,
        scope,
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
pub fn scope_can_access_route(scope: &str, request: &Request<'_>) -> bool {
    // TODO: figure out why request.segments(0..) is skipping the first segment
    let mut segments = request.uri().path().segments();

    if segments.get(0) == Some("0.8") {
        segments.next();  // skip first segment
    };

    match scope {
        "identify" => {
            request.method() == Method::Get &&
            segments.get(0) == Some("users") &&
            segments.get(1) == Some("@me")
        },
        "full" => true,
        _ => false
    }
}

#[cfg(feature = "rocket")]
pub fn scopes_can_access_route(scopes: &str, request: &Request<'_>) -> bool {
    println!("{scopes}");
    for scope in scopes.split(' ') {
        if scope_can_access_route(scope, request) {
            return true
        }
    }

    false
}