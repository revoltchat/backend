use crate::util::result::Error;

use mongodb::bson::doc;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct IdempotencyKey {
    #[validate(length(min = 1, max = 64))]
    pub key: String,
}

impl IdempotencyKey {
    // Backwards compatibility.
    // Issue #109
    pub fn consume_nonce(&mut self, v: Option<String>) {
        if let Some(v) = v {
            self.key = v;
        }
    }
}

lazy_static! {
    static ref TOKEN_CACHE: std::sync::Mutex<lru::LruCache<String, ()>> = std::sync::Mutex::new(lru::LruCache::new(100));
}

#[async_trait]
impl<'r> FromRequest<'r> for IdempotencyKey {
    type Error = Error;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(key) = request
            .headers()
            .get("Idempotency-Key")
            .next()
            .map(|k| k.to_string()) {
            let idempotency = IdempotencyKey { key };
            if let Err(error) = idempotency.validate() {
                return Outcome::Failure((Status::BadRequest, Error::FailedValidation { error }));
            }

            if let Ok(mut cache) = TOKEN_CACHE.lock() {
                if cache.get(&idempotency.key).is_some() {
                    return Outcome::Failure((Status::Conflict, Error::DuplicateNonce));
                }
        
                cache.put(idempotency.key.clone(), ());
                return Outcome::Success(idempotency);
            }
        }
        
        Outcome::Success(IdempotencyKey { key: ulid::Ulid::new().to_string() })
    }
}
