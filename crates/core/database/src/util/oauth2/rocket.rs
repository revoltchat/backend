use std::{convert::Infallible, marker::PhantomData};

use revolt_okapi::openapi3::{OAuthFlows, SecurityScheme, SecuritySchemeData};
use revolt_rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket::{request::{self, FromRequest}, Request};

use super::{OAuth2Scoped, scopes::OAuth2Scope};


#[rocket::async_trait]
impl<'r, Scope: OAuth2Scope> FromRequest<'r> for OAuth2Scoped<Scope> {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request.local_cache(|| Some(Scope::SCOPE));

        request::Outcome::Success(OAuth2Scoped { _scope: PhantomData })
    }
}

impl<'r, Scope: OAuth2Scope> OpenApiFromRequest<'r> for OAuth2Scoped<Scope> {
    fn from_request_input(
        _gen: &mut revolt_rocket_okapi::r#gen::OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> revolt_rocket_okapi::Result<revolt_rocket_okapi::request::RequestHeaderInput> {
        Ok(RequestHeaderInput::Security(
            "OAuth2".to_owned(),
            SecurityScheme {
                description: None,
                extensions: Default::default(),
                data: SecuritySchemeData::OAuth2 {
                    flows: OAuthFlows::AuthorizationCode {
                        authorization_url: "todo".to_string(),
                        token_url: "todo".to_string(),
                        refresh_url: Some("todo".to_string()),
                        scopes: revolt_okapi::map! {
                            "read:identify".to_string() => "".to_string(),
                            "read:servers".to_string() => "".to_string(),
                            "write:files".to_string() => "".to_string(),
                            "events".to_string() => "".to_string(),
                            "full".to_string() => "".to_string(),
                        },
                        extensions: Default::default()
                    }
                }
            },
            revolt_okapi::map! {
                "OAuth2".to_owned() => vec![Scope::MODEL.to_string()]
            },
        ))
    }
}
