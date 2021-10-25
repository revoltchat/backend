use rocket::{async_trait, http::Status, request::{Outcome, FromRequest}, response};
use std::{collections::{HashMap, hash_map::DefaultHasher}, time};
use crate::{database::User, util::result::Error};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

#[inline]
fn now() -> f64 {
    // will this ever actually panic?
    time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()
}

struct Ratelimit {
    pub rate: u8,
    pub per: u8,
    window: f64,
    tokens: u8,
    pub last: f64,
}

impl Ratelimit {
    pub fn new(rate: u8, per: u8) -> Self {
        Self {
            rate,
            per,
            window: 0f64,
            tokens: rate,
            last: 0.0
        }
    }

    fn get_tokens(&self, current: Option<f64>) -> u8 {
        let current = current.unwrap_or(now());
        if current > (self.window + self.per as f64) {
            self.rate
        } else {
            self.tokens
        }
    }

    fn update_ratelimit(&mut self) -> Option<f64> {
        let current = now();
        self.last = current;

        self.tokens = self.get_tokens(Some(current));

        if self.tokens == self.rate {
            self.window = current
        }

        if self.tokens == 0 {
            return Some(self.per as f64 - (current - self.window))
        }

        self.tokens -= 1;

        if self.tokens == 0 {
            self.window = current
        }

        None
    }
}

impl Clone for Ratelimit {
    fn clone(&self) -> Ratelimit {
        Ratelimit::new(self.rate, self.per)
    }
}

#[derive(Clone)]
struct RatelimitMapping {
    cache: HashMap<u64, Ratelimit>,
    cooldown: Ratelimit
}

impl RatelimitMapping {
    pub fn new(rate: u8, per: u8) -> Self {
        RatelimitMapping {
            cache: HashMap::new(),
            cooldown: Ratelimit::new(rate, per)
        }
    }

    fn verify_cache(&mut self) {
        let current = now();
        self.cache.retain(|_, v| current < v.last + v.per as f64);
    }

    pub fn get_bucket(&mut self, key: u64) -> &mut Ratelimit {
        self.verify_cache();
        self.cache.entry(key).or_insert(self.cooldown.clone())
    }
}

#[derive(Copy, Clone)]
pub struct Ratelimiter {
    bucket: u64,
    limit: u8,
    remaining: u8,
    reset: f64
}

pub struct RatelimitState(Arc<Mutex<HashMap<&'static str, RatelimitMapping>>>);

impl RatelimitState {
    pub fn new() -> Self {
        let mut hashmap = HashMap::new();
        hashmap.insert("message_send", RatelimitMapping::new(10, 10));
        let arc = Arc::new(Mutex::new(hashmap));
        RatelimitState(arc)
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for Ratelimiter {
    type Error = Error;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let res = request.local_cache_async(async {
            if let Some(route) = request.route() {
                if let Some(route_name) = &route.name {
                    let user = request.guard::<User>().await.unwrap();

                    let state = request.guard::<&rocket::State<RatelimitState>>().await.unwrap().inner();
                    let arc = Arc::clone(&state.0);
                    let mut mutex = arc.lock().unwrap();
                    let mapping = mutex.get_mut(route_name.as_ref()).unwrap();

                    // create a unique key tied to the user id and route they use
                    let key = format!("{}:{}:{}", user.id, route.method.as_str(), route.uri.as_str());
                    let mut hasher = DefaultHasher::new();

                    key.hash(&mut hasher);
                    let hashed = hasher.finish();

                    let bucket = mapping.get_bucket(hashed);
                    let ret = bucket.update_ratelimit();

                    if let Some(retry_after) = ret {
                        Err(Error::TooManyRequests { retry_after })
                    } else {
                        Ok(Ratelimiter {
                            bucket: hashed,
                            limit: bucket.rate,
                            remaining: bucket.get_tokens(None),
                            reset: bucket.window + (bucket.per as f64)
                        })
                    }
                } else {
                    unreachable!()
                }
            } else {
                unreachable!()
            }
        }).await;

        match res {
            Ok(rl) => Outcome::Success(*rl),
            Err(e) => Outcome::Failure((Status::TooManyRequests, e.clone()))
        }
    }
}

pub struct RatelimitResponse<R>(pub R);

impl<'r, 'o: 'r, R: response::Responder<'r, 'o>> response::Responder<'r, 'o> for RatelimitResponse<R> {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> response::Result<'o> {
        let res: &Result<Ratelimiter, Error> = request.local_cache(|| unreachable!());
        let ratelimiter = res.as_ref().unwrap();

        Ok(response::Builder::new(rocket::Response::new())
            .raw_header("x-ratelimit-bucket", ratelimiter.bucket.to_string())
            .raw_header("x-ratelimit-limit", ratelimiter.limit.to_string())
            .raw_header("x-ratelimit-remaining", ratelimiter.remaining.to_string())
            .raw_header("x-ratelimit-reset", ratelimiter.reset.to_string())
            .merge(self.0.respond_to(request)?)
            .finalize())
    }
}
