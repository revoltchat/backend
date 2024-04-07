#[cfg(feature = "rocket")]
use rocket::FromForm;

use std::collections::HashMap;

/// HashMap of user settings
/// Each key is mapped to a tuple consisting of the
/// revision timestamp and serialised data (in JSON format)
pub type UserSettings = HashMap<String, (i64, String)>;

auto_derived!(
    /// Options for fetching settings
    pub struct OptionsFetchSettings {
        /// Keys to fetch
        pub keys: Vec<String>,
    }

    /// Additional options for inserting settings
    #[cfg_attr(feature = "rocket", derive(FromForm))]
    pub struct OptionsSetSettings {
        /// Timestamp of settings change.
        ///
        /// Used to avoid feedback loops.
        pub timestamp: Option<i64>,
    }
);
