use revolt_quark::{EmptyResponse, Result};

#[put("/<target>/ack/<message>")]
pub async fn req(/*user: UserRef, target: Ref, message: Ref*/ target: String, message: String) -> Result<EmptyResponse> {
    todo!()
}
