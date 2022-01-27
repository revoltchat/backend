use revolt_quark::{EmptyResponse, Result};

#[delete("/<target>")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<EmptyResponse> {
    todo!()
}
