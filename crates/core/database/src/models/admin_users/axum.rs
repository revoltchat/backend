use axum::{extract::FromRequestParts, http::request::Parts};

use revolt_result::{create_error, Error, Result};

use crate::{AdminUser, Database};

#[async_trait::async_trait]
impl FromRequestParts<Database> for AdminUser {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, db: &Database) -> Result<AdminUser> {
        if let Some(Ok(token)) = parts.headers.get("x-admin-user").map(|v| v.to_str()) {
            let session = db.admin_token_authenticate(token).await?;
            db.admin_user_fetch(&session.user_id).await
        } else {
            Err(create_error!(NotAuthenticated))
        }
    }
}
