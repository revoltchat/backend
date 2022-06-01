use std::collections::HashMap;

/// HashMap of user settings
/// Each key is mapped to a tuple consisting of the
/// revision timestamp and serialised data (in JSON format)
pub type UserSettings = HashMap<String, (i64, String)>;
