use crate::v0::{PublicBot, User};
use std::collections::HashMap;

auto_derived!(
    pub struct OAuth2AuthorizeAuthResponse {
        /// Redirect URI which will contain the access token
        pub redirect_uri: String,
    }

    pub struct OAuth2ScopeReasoning {
        pub allow: String,
        pub deny: String
    }

    pub struct OAuth2AuthorizeInfoResponse {
        pub bot: PublicBot,
        pub user: User,
        pub allowed_scopes: HashMap<OAuth2Scope, OAuth2ScopeReasoning>
    }

    #[derive(Copy)]
    #[cfg_attr(feature = "serde", serde(rename = "lowercase"))]
    #[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
    pub enum OAuth2ResponseType {
        #[cfg_attr(feature = "rocket", field(value = "code"))]
        Code,
        #[cfg_attr(feature = "rocket", field(value = "token"))]
        Token,
    }

    #[derive(Copy)]
    #[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
    pub enum OAuth2GrantType {
        #[cfg_attr(feature = "rocket", field(value = "authorization_code"))]
        #[cfg_attr(feature = "serde", serde(rename = "authorization_code"))]
        AuthorizationCode,
        #[cfg_attr(feature = "rocket", field(value = "implicit"))]
        #[cfg_attr(feature = "serde", serde(rename = "implicit"))]
        Implicit,
    }

    #[derive(Copy)]
    #[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
    pub enum OAuth2CodeChallangeMethod {
        #[cfg_attr(feature = "rocket", field(value = "plain"))]
        #[cfg_attr(feature = "serde", serde(rename = "plain"))]
        Plain,
        S256
    }

    #[cfg_attr(feature = "rocket", derive(rocket::FromForm))]
    pub struct OAuth2AuthorizationForm {
        pub client_id: String,
        pub scope: String,
        pub redirect_uri: String,
        pub response_type: OAuth2ResponseType,
        pub state: Option<String>,
        pub code_challenge: Option<String>,
        pub code_challenge_method: Option<OAuth2CodeChallangeMethod>,
    }

    #[cfg_attr(feature = "rocket", derive(rocket::FromForm))]
    pub struct OAuth2TokenExchangeForm {
        pub grant_type: OAuth2GrantType,

        pub client_id: String,
        pub client_secret: Option<String>,

        pub code: String,
        pub code_verifier: Option<String>,
    }

    pub struct OAuth2TokenExchangeResponse {
        pub access_token: String,
        pub token_type: String,
        pub scope: String,
    }

    #[derive(Copy, Hash)]
    #[cfg_attr(feature = "serde", serde(rename = "lowercase"))]
    pub enum OAuth2Scope {
        Identify,
        Full,
    }
);

impl OAuth2Scope {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Option<OAuth2Scope> {
        match str {
            "identify" => Some(OAuth2Scope::Identify),
            "full" => Some(OAuth2Scope::Full),
            _ => None,
        }
    }

    pub fn scopes_from_str(string: &str) -> Option<Vec<OAuth2Scope>> {
        let mut scopes = Vec::new();

        for scope in string.split(' ') {
            if let Some(scope) = OAuth2Scope::from_str(scope) {
                scopes.push(scope);
            } else {
                return None;
            }
        }

        Some(scopes)
    }
}
