use revolt_quark::{Error, Result};

use mongodb::bson::doc;
use rocket::serde::json::Value;

#[get("/<target>/members/<member>")]
pub async fn req(
    /*user: UserRef, target: Ref,*/ target: String,
    member: String,
) -> Result<Value> {
    todo!()
}
