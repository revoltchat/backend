use revolt_quark::{EmptyResponse, Result};

use mongodb::bson::doc;
use rauth::entities::{Model, Session, WebPushSubscription};
use rocket::serde::json::Json;

#[post("/subscribe"/*, data = "<data>"*/)]
pub async fn req(/*mut session: Session, data: Json<WebPushSubscription>*/) -> Result<EmptyResponse>
{
    todo!()
}
