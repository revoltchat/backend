use revolt_quark::{Db, Ref, Result, models::{Webhook, File}};
use rocket::serde::json::Json;
use serde::{Serialize, Deserialize};


// This route is used to get the info about the webhook by clients to get the name and avatar,
// so this function cant return the token or require any permissions.

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct WebhookData {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,
    pub channel: String,
}

/// # gets a webhook
///
/// gets a webhook
#[openapi(tag = "Webhooks")]
#[get("/<target>")]
pub async fn req(db: &Db, target: Ref) -> Result<Json<WebhookData>> {
    let Webhook { id, name, avatar, channel, .. } = target.as_webhook(db).await?;

    Ok(Json(WebhookData { id, name, avatar, channel }))
}
