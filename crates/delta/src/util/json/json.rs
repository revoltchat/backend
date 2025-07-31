use std::fmt::Debug;

use revolt_rocket_okapi::{
    r#gen::OpenApiGenerator,
    request::OpenApiFromData,
    response::OpenApiResponderInner,
    revolt_okapi::openapi3::{RequestBody, Responses}, util::add_schema_response
};
use rocket::{data::{Data, FromData, Limits, Outcome}};
use rocket::response::{self, Responder, content};
use rocket::request::{local_cache, Request};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use revolt_result::{create_error, Error, ToRevoltError};

// A lot of this code is modified versions of rocket::serde::json so we can store
// the error so it can be passed to the error catcher.

#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<'r, T: Deserialize<'r>> Json<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    fn from_str(s: &'r str) -> Result<Self, Error> {
        serde_json::from_str(s)
            .map(Json)
            .map_err(|e| create_error!(DeserializationError { error: e.to_string() }))
    }

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Result<Self, Error> {
        let limit = req.limits().get("json").unwrap_or(Limits::JSON);
        let string = match data.open(limit).into_string().await {
            Ok(s) if s.is_complete() => s.into_inner(),
            Ok(_) => {
                return Err(create_error!(PayloadTooLarge));
            },
            Err(_) => return Err(create_error!(IOError)),
        };

        Self::from_str(local_cache!(req, string))
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<'r, T: Deserialize<'r> + std::fmt::Debug> FromData<'r> for Json<T> {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let r = Self::from_data(req, data).await;

        match r {
            Ok(value) => Outcome::Success(value),
            Err(e) => {
                req.local_cache(|| Some(e.clone()));
                rocket::outcome::Outcome::Error((e.rocket_status(), e))
            }
        }
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for Json<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match serde_json::to_string(&self.0).capture_error() {
            Ok(string) => content::RawJson(string).respond_to(req),
            Err(_) => create_error!(InternalError).respond_to(req)
        }
    }
}

impl<'r, T: JsonSchema + Deserialize<'r> + Debug> OpenApiFromData<'r> for Json<T> {
    fn request_body(gen: &mut OpenApiGenerator) -> revolt_rocket_okapi::Result<RequestBody> {
        crate::fn_request_body!(gen, T, "application/json")
    }
}

impl<T: JsonSchema + Serialize> OpenApiResponderInner for Json<T> {
    fn responses(gen: &mut OpenApiGenerator) -> revolt_rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<T>();
        add_schema_response(&mut responses, 200, "application/json", schema)?;
        Ok(responses)
    }
}

impl<T: validator::Validate> validator::Validate for Json<T> {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        self.0.validate()
    }
}
