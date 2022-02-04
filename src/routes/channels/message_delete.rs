use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;

#[delete("/<target>/messages/<msg>")]
pub async fn req(
    /*user: UserRef, target: Ref, msg: Ref*/ target: String,
    msg: String,
) -> Result<EmptyResponse> {
    todo!()
}
