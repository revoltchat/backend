use revolt_quark::Result;

use serde_json::Value;

#[get("/<target>/invite")]
pub async fn fetch_public_bot(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
