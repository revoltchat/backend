use std::marker::PhantomData;

use axum::{extract::FromRequestParts, http::request::Parts};
use revolt_result::{Error, Result};

use super::{OAuth2Scoped, scopes::OAuth2Scope};

#[rocket::async_trait]
impl<S: Send + Sync, Scope: OAuth2Scope> FromRequestParts<S> for OAuth2Scoped<Scope> {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts.extensions.insert(Scope::SCOPE);

        Ok(OAuth2Scoped { _scope: PhantomData })
    }
}