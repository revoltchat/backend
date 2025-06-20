use revolt_result::Result;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

use crate::{AdminMachineToken, Database};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminMachineToken {
    type Error = authifier::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user: &Option<AdminMachineToken> = request
            .local_cache_async(async {
                let db = request.rocket().state::<Database>().expect("`Database`");

                if let Some(on_behalf_of) = request
                    .headers()
                    .get("x-admin-on-behalf-of")
                    .next()
                    .map(|x| x.to_string())
                {
                    if let Some(token) = request
                        .headers()
                        .get("x-admin-machine")
                        .next()
                        .map(|x| x.to_string())
                    {
                        let config = revolt_config::config().await;
                        let token = token.to_string();
                        if config.api.security.admin_keys.contains(&token) {
                            let resp: Result<AdminMachineToken> = if on_behalf_of.contains("@") {
                                AdminMachineToken::new_from_email(&on_behalf_of, db).await
                            } else {
                                AdminMachineToken::new_from_id(&on_behalf_of, db).await
                            };
                            if let Ok(resp) = resp {
                                return Some(resp);
                            }
                        }
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
