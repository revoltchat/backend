use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref REVCORD_URL: String = env::var("REVCORD_URL").expect("Missing REVCORD_URL environment variable");
}
