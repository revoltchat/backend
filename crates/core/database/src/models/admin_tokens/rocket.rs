use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

use crate::AdminMachineToken;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminMachineToken {
    type Error = authifier::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user: &Option<AdminMachineToken> = request
            .local_cache_async(async {
                let config = revolt_config::config().await;

                let token = request
                    .headers()
                    .get("x-admin-machine")
                    .next()
                    .map(|x| x.to_string());

                if let Some(token) = token {
                    if config.api.security.admin_keys.contains(&token) {
                        return Some(AdminMachineToken::new());
                    }
                }

                None
            })
            .await;

        if let Some(user) = user {
            Outcome::Success(user.clone())
        } else {
            Outcome::Error((Status::Unauthorized, authifier::Error::InvalidSession))
        }
    }
}
