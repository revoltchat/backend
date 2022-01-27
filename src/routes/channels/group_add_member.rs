use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;

#[put("/<target>/recipients/<member>")]
pub async fn req(/*user: UserRef, target: Ref, member: Ref*/ target: String, member: String) -> Result<EmptyResponse> {
    todo!()
}
