use revolt_quark::{models::message::Masquerade, Result};

use mongodb::bson::doc;
use regex::Regex;
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct Reply {
    id: String,
    mention: bool,
}

#[derive(Validate, Serialize, Deserialize, Clone, Debug)]
pub struct SendableEmbed {
    icon_url: Option<String>,
    url: Option<String>,
    #[validate(length(min = 1, max = 100))]
    title: Option<String>,
    #[validate(length(min = 1, max = 2000))]
    description: Option<String>,
    media: Option<String>,
    colour: Option<String>,
}
#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 0, max = 2000))]
    content: String,
    #[validate(length(min = 1, max = 128))]
    attachments: Option<Vec<String>>,
    nonce: Option<String>,
    replies: Option<Vec<Reply>>,
    //#[validate]
    masquerade: Option<Masquerade>,
    #[validate(length(min = 1, max = 10))]
    embeds: Option<Vec<SendableEmbed>>,
}

lazy_static! {
    // ignoring I L O and U is intentional
    static ref RE_MENTION: Regex = Regex::new(r"<@([0-9A-HJKMNP-TV-Z]{26})>").unwrap();
}

#[post("/<target>/messages", data = "<message>")]
pub async fn message_send(
    /*user: UserRef, _r: Ratelimiter, mut idempotency: IdempotencyKey, target: Ref,*/
    target: String,
    message: Json<Data>,
) -> Result<Value> {
    todo!()
}
