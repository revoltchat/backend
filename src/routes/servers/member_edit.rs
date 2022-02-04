use revolt_quark::{models::server_member::FieldsMember, EmptyResponse, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 1, max = 32))]
    nickname: Option<String>,
    avatar: Option<String>,
    roles: Option<Vec<String>>,
    remove: Option<FieldsMember>,
}

#[patch("/<server>/members/<target>", data = "<data>")]
pub async fn req(
    /*user: UserRef, server: Ref,*/ server: String,
    target: String,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    todo!()
}
