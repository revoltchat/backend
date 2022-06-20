use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for valid usernames
///
/// Block zero width space
/// Block lookalike characters
pub static RE_USERNAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^\u200BА-Яа-яΑ-Ωα-ω@#:\n]+$").unwrap());
