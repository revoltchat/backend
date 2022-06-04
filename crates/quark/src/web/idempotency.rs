use crate::{Error, Result};

use async_std::sync::Mutex;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{Parameter, ParameterValue};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use schemars::schema::{InstanceType, SchemaObject, SingleOrVec};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct IdempotencyKey {
    #[validate(length(min = 1, max = 64))]
    key: String,
}

lazy_static! {
    static ref TOKEN_CACHE: Mutex<lru::LruCache<String, ()>> = Mutex::new(lru::LruCache::new(100));
}

impl IdempotencyKey {
    // Backwards compatibility.
    // Issue #109
    pub async fn consume_nonce(&mut self, v: Option<String>) -> Result<()> {
        if let Some(v) = v {
            let mut cache = TOKEN_CACHE.lock().await;
            if cache.get(&v).is_some() {
                return Err(Error::DuplicateNonce);
            }

            cache.put(v.clone(), ());
            self.key = v;
        }

        Ok(())
    }

    pub fn into_key(self) -> String {
        self.key
    }
}

impl<'r> OpenApiFromRequest<'r> for IdempotencyKey {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::Parameter(Parameter {
            name: "Idempotency-Key".to_string(),
            description: Some("Unique key to prevent duplicate requests".to_string()),
            allow_empty_value: false,
            required: false,
            deprecated: false,
            extensions: schemars::Map::new(),
            location: "header".to_string(),
            value: ParameterValue::Schema {
                allow_reserved: false,
                example: None,
                examples: None,
                explode: None,
                style: None,
                schema: SchemaObject {
                    instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                    ..Default::default()
                },
            },
        }))
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for IdempotencyKey {
    type Error = Error;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(key) = request
            .headers()
            .get("Idempotency-Key")
            .next()
            .map(|k| k.to_string())
        {
            let idempotency = IdempotencyKey { key };
            if let Err(error) = idempotency.validate() {
                return Outcome::Failure((Status::BadRequest, Error::FailedValidation { error }));
            }

            let mut cache = TOKEN_CACHE.lock().await;
            if cache.get(&idempotency.key).is_some() {
                return Outcome::Failure((Status::Conflict, Error::DuplicateNonce));
            }

            cache.put(idempotency.key.clone(), ());
            return Outcome::Success(idempotency);
        }

        Outcome::Success(IdempotencyKey {
            key: ulid::Ulid::new().to_string(),
        })
    }
}
