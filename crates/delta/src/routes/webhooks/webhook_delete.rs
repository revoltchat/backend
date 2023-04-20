use revolt_quark::{Db, Ref, Result, EmptyResponse, models::User, perms, Permission};

/// # Deletes a webhook
///
/// deletes a webhook
#[openapi(tag = "Webhooks")]
#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let webhook = target.as_webhook(db).await?;

    let channel = Ref::from_unchecked(webhook.channel.clone()).as_channel(db).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission(db, Permission::ManageWebhooks)
        .await?;

    webhook.delete(db).await?;

    Ok(EmptyResponse)
}
