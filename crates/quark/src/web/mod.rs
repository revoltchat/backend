use crate::Database;
use rocket::State;

pub use rocket_empty::EmptyResponse;
pub type Db = State<Database>;
