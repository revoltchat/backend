use revolt_quark::models::User;
use revolt_quark::{EmptyResponse, Result, Db};

use chrono::prelude::*;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Data = HashMap<String, String>;

#[derive(FromForm, Serialize, Deserialize)]
pub struct Options {
    timestamp: Option<i64>,
}

#[post("/settings/set?<options..>", data = "<data>")]
pub async fn req(db: &Db, user: User, data: Json<Data>, options: Options) -> Result<EmptyResponse> {
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
        settings.insert(key, (
            timestamp,
            data
        ));
    }

    db.set_user_settings(&user.id, &settings).await.map(|_| EmptyResponse)
}
