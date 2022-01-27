use revolt_quark::{EmptyResponse, Result, models::server::{FieldsServer, SystemMessageChannels, Category}};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,
    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    categories: Option<Vec<Category>>,
    system_messages: Option<SystemMessageChannels>,
    remove: Option<FieldsServer>,
    nsfw: Option<bool>,
    analytics: Option<bool>,
}

#[patch("/<target>", data = "<data>")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, data: Json<Data>) -> Result<EmptyResponse> {
    todo!()
}
