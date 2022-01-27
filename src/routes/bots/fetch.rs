use revolt_quark::Result;

use serde_json::Value;

#[get("/<target>")]
pub async fn fetch_bot(/*user: UserRef, target: Ref*/ target: String) -> Result<Value> {
    todo!()
}
