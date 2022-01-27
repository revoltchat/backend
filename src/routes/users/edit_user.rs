use revolt_quark::{EmptyResponse, Result, models::user::FieldsUser};

use mongodb::bson::{doc, to_document};
use revolt_quark::models::user::UserStatus;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct UserProfileData {
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 128))]
    background: Option<String>,
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    // #[validate]
    status: Option<UserStatus>,
    #[validate]
    profile: Option<UserProfileData>,
    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,
    remove: Option<FieldsUser>,
}

#[patch("/<_ignore_id>", data = "<data>")]
pub async fn req(/*user: UserRef,*/ data: Json<Data>, _ignore_id: String) -> Result<EmptyResponse> {
    todo!()
}
