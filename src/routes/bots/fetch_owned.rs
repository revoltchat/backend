use revolt_quark::Result;

use serde_json::Value;

#[get("/@me")]
pub async fn fetch_owned_bots(/*user: UserRef*/) -> Result<Value> {
    todo!()
}
