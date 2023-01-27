use revolt_quark::{Db, Ref, Result, Error, models::Webhook};
use rocket::serde::json::Json;

/// # gets a webhook
///
/// gets a webhook
#[openapi(tag = "Webhooks")]
#[get("/<target>/<token>")]
pub async fn req(db: &Db, target: Ref, token: String) -> Result<Json<Webhook>> {
    let webhook = target.as_webhook(db).await?;

    (webhook.token == token)
        .then_some(())
        .ok_or(Error::InvalidCredentials)?;

    Ok(Json(webhook))
}
