use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use chrono::prelude::*;
use std::collections::HashMap;
use rocket_contrib::json::Json;
use mongodb::bson::{doc, to_bson};
use mongodb::options::UpdateOptions;

type Data = HashMap<String, String>;

#[post("/settings/set", data = "<data>")]
pub async fn req(user: User, data: Json<Data>) -> Result<()> {
    let data = data.into_inner();

    let mut set = doc! {};
    for (key, data) in &data {
        set.insert(
            key.clone(),
            vec! [
                to_bson(&Utc::now().timestamp_millis()).unwrap(),
                to_bson(&data.clone()).unwrap()
            ]
        );
    }

    if set.len() > 0 {
        get_collection("user_settings")
            .update_one(
                doc! {
                    "_id": &user.id
                },
                doc! {
                    "$set": set
                },
                UpdateOptions::builder()
                    .upsert(true)
                    .build()
            )
            .await
            .map_err(|_| Error::DatabaseError { operation: "update_one", with: "user_settings" })?;
    }

    ClientboundNotification::UserSettingsUpdate {
        id: user.id.clone(),
        update: data
    }
    .publish(user.id);
    
    Ok(())
}
