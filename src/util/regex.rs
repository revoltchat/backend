use once_cell::sync::Lazy;
use regex::Regex;

pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.]+$").unwrap());
