use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for valid display names
///
/// Block zero width space
/// Block newline and carriage return
pub static RE_DISPLAY_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^\u200B\n\r]+$").unwrap());

/// Regex for valid usernames
///
/// Block zero width space
/// Block lookalike characters
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.-])+$").unwrap());

/// Regex for valid emoji names
///
/// Alphanumeric and underscores
pub static RE_EMOJI: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9_]+$").unwrap());
