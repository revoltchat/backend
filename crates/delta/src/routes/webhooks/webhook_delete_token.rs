use revolt_quark::{Db, Ref, Result, EmptyResponse, Error};

/// # Deletes a webhook
///
/// deletes a webhook with a token
#[openapi(tag = "Webhooks")]
#[delete("/<target>/<token>")]
pub async fn req(db: &Db, target: Ref, token: String) -> Result<EmptyResponse> {
    let webhook = target.as_webhook(db).await?;

    (webhook.token.as_deref() == Some(&token))
        .then_some(())
        .ok_or(Error::InvalidCredentials)?;

    webhook.delete(db).await?;

    Ok(EmptyResponse)
}
