use revolt_quark::{EmptyResponse, Result};

use rocket::serde::json::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerId {
    server: String,
}

#[derive(Deserialize)]
pub struct GroupId {
    group: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Destination {
    Server(ServerId),
    Group(GroupId),
}

#[post("/<target>/invite", data = "<dest>")]
pub async fn invite_bot(
    /*user: UserRef, target: Ref,*/ target: String,
    dest: Json<Destination>,
) -> Result<EmptyResponse> {
    todo!()
}
