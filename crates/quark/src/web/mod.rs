use crate::Database;
use rocket::State;

pub mod cors;
pub mod idempotency;
pub mod ratelimiter;
pub mod swagger;

pub use rocket_empty::EmptyResponse;
pub type Db = State<Database>;
