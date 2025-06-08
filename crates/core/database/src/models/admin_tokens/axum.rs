use axum::{extract::FromRequestParts, http::request::Parts};

use revolt_result::{create_error, Error, Result};

use crate::{AdminMachineToken, Database};

#[async_trait::async_trait]
impl FromRequestParts<Database> for AdminMachineToken {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _db: &Database) -> Result<AdminMachineToken> {
        if let Some(Ok(token)) = parts.headers.get("x-admin-machine").map(|v| v.to_str()) {
            let config = revolt_config::config().await;
            let token = token.to_string();
            if config.api.security.admin_keys.contains(&token) {
                Ok(AdminMachineToken::new())
            } else {
                Err(create_error!(InvalidCredentials))
            }
        } else {
            Err(create_error!(NotAuthenticated))
        }
    }
}
