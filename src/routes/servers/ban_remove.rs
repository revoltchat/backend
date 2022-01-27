use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;

#[delete("/<server>/bans/<target>")]
pub async fn req(/*user: UserRef, server: Ref, target: Ref*/ server: String, target: String) -> Result<EmptyResponse> {
    todo!()
}
