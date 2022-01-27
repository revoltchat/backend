use revolt_quark::{EmptyResponse, Result};

#[delete("/<target>/roles/<role_id>")]
pub async fn req(/*user: UserRef, target: Ref,*/ target: String, role_id: String) -> Result<EmptyResponse> {
    todo!()
}
