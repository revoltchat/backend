use revolt_quark::{Db, Ref, Result, Error, models::Webhook};
use rocket::serde::json::Json;

/// # Gets a webhook
///
/// gets a webhook with a token
#[openapi(tag = "Webhooks")]
#[get("/<target>/<token>")]
pub async fn webhook_fetch_token(db: &Db, target: Ref, token: String) -> Result<Json<Webhook>> {
    let webhook = target.as_webhook(db).await?;

    (webhook.token.as_deref() == Some(&token))
        .then_some(())
        .ok_or(Error::InvalidCredentials)?;

    Ok(Json(webhook))
}
