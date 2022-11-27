use revolt_quark::{Db, Ref, Result, Error, models::{Webhook, Message, message::SendableEmbed}, types::push::MessageAuthor};
use rocket::{serde::{json::Json, DeserializeOwned}, Request, request::FromRequest, http::Status};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ulid::Ulid;


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Star {
        action: String,
        sender: Value
    }
}

struct EventHeader<'r>(pub &'r str);

#[async_trait]
impl<'r> FromRequest<'r> for EventHeader<'r> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self,Self::Error> {
        let headers = request.headers();
        let Some(event) = headers.get_one("X-GitHub-Event") else {
            return rocket::request::Outcome::Failure((Status::BadRequest, Error::InvalidOperation))
        };

        rocket::request::Outcome::Success(Self(event))
    }
}

/// # executes a webhook specific to github
///
/// executes a webhook specific to github and sends a message containg the relavent info about the event
#[openapi(tag = "Webhooks")]
#[post("/<target>/<token>/github", data="<data>")]
pub async fn req(db: &Db, target: Ref, token: String, event: EventHeader<'_>, data: String) -> Result<()> {
    let webhook = target.as_webhook(db).await?;

    (webhook.token == token)
        .then_some(())
        .ok_or(Error::InvalidCredentials)?;

    let channel = db.fetch_channel(&webhook.channel).await?;

    let body = format!(r#"{{"{}": {data}}}"#r, event.0);

    let Ok(event) = serde_json::from_str::<Event>(&body) else {
        return Err(Error::InvalidOperation)
    };

    let sendable_embed = match event {
        Event::Star { action, sender } => SendableEmbed {
            title: Some(format!("{action} star")),
            ..Default::default()
        },
    };

    let message_id = Ulid::new().to_string();

    let embed = sendable_embed.into_embed(db, message_id.clone()).await?;

    let message = Message {
        id: message_id,
        channel: webhook.channel,
        embeds: Some(vec![embed]),
        webhook: Some(webhook.id.clone()),
        ..Default::default()
    };

    message.create(db, &channel, Some(MessageAuthor::Webhook(&webhook))).await;

    Ok(())
}
