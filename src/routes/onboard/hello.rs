use crate::util::result::Result;
use rauth::auth::Session;

#[get("/hello")]
pub async fn req(session: Session) -> Result<String> {
    Ok("try onboard user".to_string())
}
