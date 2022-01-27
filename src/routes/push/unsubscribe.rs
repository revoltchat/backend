use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;
use rauth::entities::{Model, Session};

#[post("/unsubscribe")]
pub async fn req(/*mut session: Session*/) -> Result<EmptyResponse> {
    todo!()
}
