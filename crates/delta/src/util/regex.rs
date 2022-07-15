use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for valid usernames
///
/// Block zero width space
/// Block lookalike characters
pub static RE_USERNAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^\u200BА-Яа-яΑ-Ωα-ω@#:\n\r\[\]]+$").unwrap());

/// Regex for valid emoji names
///
/// Alphanumeric and underscores
pub static RE_EMOJI: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9_]+$").unwrap());
