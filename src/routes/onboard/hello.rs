use rauth::entities::Session;
use rocket::serde::json::Value;

#[get("/hello")]
pub async fn req(/*_session: Session, user: Option<User>*/) -> Value {
    todo!()
}
