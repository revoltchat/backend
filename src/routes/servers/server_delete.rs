use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;

#[delete("/<target>")]
pub async fn req(/*user: UserRef, target: Ref*/ target: String) -> Result<EmptyResponse> {
    todo!()
}
