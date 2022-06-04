use revolt_quark::models::User;
use revolt_quark::r#impl::UserSettingsImpl;
use revolt_quark::{Db, EmptyResponse, Result};

use chrono::prelude::*;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Data = HashMap<String, String>;

/// # Set Options
#[derive(FromForm, Serialize, Deserialize, JsonSchema)]
pub struct OptionsSetSettings {
    /// Timestamp of settings change.
    ///
    /// Used to avoid feedback loops.
    timestamp: Option<i64>,
}

/// # Set Settings
///
/// Upload data to save to settings.
#[openapi(tag = "Sync")]
#[post("/settings/set?<options..>", data = "<data>")]
pub async fn req(
    db: &Db,
    user: User,
    data: Json<Data>,
    options: OptionsSetSettings,
) -> Result<EmptyResponse> {
    let data = data.into_inner();
    let current_time = Utc::now().timestamp_millis();
    let timestamp = if let Some(timestamp) = options.timestamp {
        if timestamp > current_time {
            current_time
        } else {
            timestamp
        }
    } else {
        current_time
    };

    let mut settings = HashMap::new();
    for (key, data) in data {
        settings.insert(key, (timestamp, data));
    }

    settings.set(db, &user.id).await.map(|_| EmptyResponse)
}
