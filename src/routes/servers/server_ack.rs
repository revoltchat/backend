use revolt_quark::{EmptyResponse, Result};

#[put("/<target>/ack")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<EmptyResponse> {
    todo!()
}
