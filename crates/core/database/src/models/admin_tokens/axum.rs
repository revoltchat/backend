use axum::{extract::FromRequestParts, http::request::Parts};

use revolt_result::{create_error, Error, Result};

use crate::{AdminMachineToken, Database};

#[async_trait::async_trait]
impl FromRequestParts<Database> for AdminMachineToken {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, db: &Database) -> Result<AdminMachineToken> {
        if let Some(Ok(on_behalf_of)) = parts
            .headers
            .get("x-admin-on-behalf-of")
            .map(|v| v.to_str())
        {
            if let Some(Ok(token)) = parts.headers.get("x-admin-machine").map(|v| v.to_str()) {
                let config = revolt_config::config().await;
                let token = token.to_string();
                if config.api.security.admin_keys.contains(&token) {
                    let resp: AdminMachineToken;
                    // shitty email check
                    if on_behalf_of.contains("@") {
                        resp = AdminMachineToken::new_from_email(on_behalf_of, db).await?
                    } else {
                        resp = AdminMachineToken::new_from_id(on_behalf_of, db).await?
                    }
                    return Ok(resp);
                } else {
                    return Err(create_error!(InvalidCredentials));
                }
            }
        }
        Err(create_error!(NotAuthenticated))
    }
}
