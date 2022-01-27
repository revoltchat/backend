use revolt_quark::{EmptyResponse, Result};

use chrono::Utc;
use mongodb::bson::{Bson, Document, doc, to_document};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 2000))]
    content: Option<String>,
    // #[validate(length(min = 0, max = 10))]
    // embeds: Option<Vec<SendableEmbed>>
}

#[patch("/<target>/messages/<msg>", data = "<edit>")]
pub async fn req(/*user: UserRef, target: Ref, msg: Ref,*/ target: String, msg: String, edit: Json<Data>) -> Result<EmptyResponse> {
    todo!()
}
