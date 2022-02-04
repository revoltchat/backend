use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;

#[delete("/<target>/members/<member>")]
pub async fn req(
    /*user: UserRef, target: Ref,*/ target: String,
    member: String,
) -> Result<EmptyResponse> {
    todo!()
}
