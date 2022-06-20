//! Pulled from lightspeed-tv/backend.
//!
//! This will be replaced again in the near future since
//! I don't want duplication between two different projects.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::rauth::models::Session;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::http::{Method, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Data, Request, Response};

use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};

use serde::Serialize;

use dashmap::DashMap;

/// Ratelimit Bucket
#[derive(Clone, Copy)]
struct Entry {
    used: u8,
    reset: u128,
}

lazy_static! {
    static ref MAP: DashMap<u64, Entry> = DashMap::new();
}

/// Get the current time from Unix Epoch as a Duration
fn now() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards...")
}

impl Entry {
    /// Find bucket by its key
    pub fn from(key: u64) -> Entry {
        MAP.get(&key).map(|x| *x).unwrap_or_else(|| Entry {
            used: 0,
            reset: now().add(Duration::from_secs(10)).as_millis(),
        })
    }

    /// Deduct one unit from the bucket and save
    pub fn deduct(mut self, key: u64) {
        let current_time = now().as_millis();
        if current_time > self.reset {
            self.used = 1;
            self.reset = now().add(Duration::from_secs(10)).as_millis();
        } else {
            self.used += 1;
        }

        MAP.insert(key, self);
    }

    /// Get remaining units in the bucket
    pub fn get_remaining(&self, limit: u8) -> u8 {
        if now().as_millis() > self.reset {
            limit
        } else {
            limit - self.used
        }
    }

    /// Get how long bucket has until reset
    pub fn left_until_reset(&self) -> u128 {
        let current_time = now().as_millis();
        if current_time > self.reset {
            0
        } else {
            self.reset - current_time
        }
    }
}

/// Ratelimit Guard
#[derive(Serialize, Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Ratelimiter {
    key: u64,
    limit: u8,
    remaining: u8,
    reset: u128,
}

/// Find bucket from given request
///
/// Optionally, include a resource id to hash against.
fn resolve_bucket<'r>(request: &'r rocket::Request<'_>) -> (&'r str, Option<&'r str>) {
    if let Some(segment) = request.routed_segment(0) {
        let resource = request.routed_segment(1);
        match (segment, resource) {
            ("users", _) => {
                if let Some("default_avatar") = request.routed_segment(2) {
                    return ("default_avatar", None);
                }

                ("users", None)
            }
            ("bots", _) => ("bots", None),
            ("channels", Some(id)) => {
                if request.method() == Method::Post {
                    if let Some("messages") = request.routed_segment(2) {
                        return ("messaging", Some(id));
                    }
                }

                ("channels", Some(id))
            }
            ("servers", Some(id)) => ("servers", Some(id)),
            ("auth", _) => {
                if request.method() == Method::Delete {
                    ("auth_delete", None)
                } else {
                    ("auth", None)
                }
            }
            ("swagger", _) => ("swagger", None),
            _ => ("any", None),
        }
    } else {
        ("any", None)
    }
}

/// Resolve per-bucket limits
fn resolve_bucket_limit(bucket: &str) -> u8 {
    match bucket {
        "users" => 20,
        "bots" => 10,
        "messaging" => 10,
        "channels" => 15,
        "servers" => 5,
        "auth" => 15,
        "auth_delete" => 255,
        "default_avatar" => 255,
        "swagger" => 100,
        _ => 20,
    }
}

/// Find the remote IP of the client
fn to_ip(request: &'_ rocket::Request<'_>) -> String {
    request
        .remote()
        .map(|x| x.ip().to_string())
        .unwrap_or_default()
}

/// Find the actual IP of the client
fn to_real_ip(request: &'_ rocket::Request<'_>) -> String {
    if let Ok(true) = std::env::var("TRUST_CLOUDFLARE").map(|x| x == "1") {
        request
            .headers()
            .get_one("CF-Connecting-IP")
            .map(|x| x.to_string())
            .unwrap_or_else(|| to_ip(request))
    } else {
        to_ip(request)
    }
}

impl Ratelimiter {
    /// Generate guard from identifier and target bucket
    pub fn from(
        identifier: &str,
        (bucket, resource): (&str, Option<&str>),
    ) -> Result<Ratelimiter, u128> {
        let mut key = DefaultHasher::new();
        key.write(identifier.as_bytes());
        key.write(bucket.as_bytes());

        if let Some(id) = resource {
            key.write(id.as_bytes());
        }

        let key = key.finish();
        let limit = resolve_bucket_limit(bucket);
        let entry = Entry::from(key);

        let remaining = entry.get_remaining(limit);
        let reset = entry.left_until_reset();

        if remaining > 0 {
            entry.deduct(key);
            Ok(Ratelimiter {
                key,
                limit,
                remaining: remaining - 1,
                reset,
            })
        } else {
            Err(reset)
        }
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for Ratelimiter {
    type Error = u128;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let ratelimiter = request
            .local_cache_async(async {
                use rocket::outcome::Outcome;
                let identifier = if let Outcome::Success(session) = request.guard::<Session>().await
                {
                    session.id
                } else {
                    to_real_ip(request)
                };

                Ratelimiter::from(&identifier, resolve_bucket(request))
            })
            .await;

        match ratelimiter {
            Ok(ratelimiter) => Outcome::Success(*ratelimiter),
            Err(retry_after) => Outcome::Failure((Status::TooManyRequests, *retry_after)),
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for Ratelimiter {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

/// Attach ratelimiter to the Rocket application
pub struct RatelimitFairing;

#[rocket::async_trait]
impl Fairing for RatelimitFairing {
    fn info(&self) -> Info {
        Info {
            name: "Ratelimiter",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        use rocket::outcome::Outcome;
        if let Outcome::Failure(_) = request.guard::<Ratelimiter>().await {
            info!(
                "User rate-limited on route {}! (IP = {:?})",
                request.uri(),
                to_real_ip(request)
            );

            request.set_method(Method::Get);
            request.set_uri(Origin::parse("/ratelimit").unwrap())
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        use rocket::outcome::Outcome;
        match request.guard::<Ratelimiter>().await {
            Outcome::Success(ratelimiter) => {
                let Ratelimiter {
                    key,
                    limit,
                    remaining,
                    reset,
                } = ratelimiter;

                response.set_raw_header("X-RateLimit-Limit", limit.to_string());
                response.set_raw_header("X-RateLimit-Bucket", key.to_string());
                response.set_raw_header("X-RateLimit-Remaining", remaining.to_string());
                response.set_raw_header("X-RateLimit-Reset-After", reset.to_string());
            }
            Outcome::Failure(_) => response.set_status(Status::TooManyRequests),
            Outcome::Forward(_) => unreachable!(),
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum RatelimitInformation {
    Success(Ratelimiter),
    Failure { retry_after: u128 },
}

#[async_trait]
impl<'r> FromRequest<'r> for RatelimitInformation {
    type Error = u128;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(match request.guard::<Ratelimiter>().await {
            Outcome::Success(ratelimiter) => RatelimitInformation::Success(ratelimiter),
            Outcome::Failure((_, retry_after)) => RatelimitInformation::Failure { retry_after },
            _ => unreachable!(),
        })
    }
}

#[rocket::get("/ratelimit")]
fn ratelimit_info(info: RatelimitInformation) -> Json<RatelimitInformation> {
    Json(info)
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![ratelimit_info]
}
