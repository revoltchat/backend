use crate::util::regex::RE_USERNAME;

use revolt_quark::{models::bot::FieldsBot, EmptyResponse, Result};

use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    public: Option<bool>,
    analytics: Option<bool>,
    interactions_url: Option<String>,
    remove: Option<FieldsBot>,
}

#[patch("/<target>", data = "<data>")]
pub async fn edit_bot(
    /*user: UserRef, target: Ref,*/ target: String,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    todo!()
}
