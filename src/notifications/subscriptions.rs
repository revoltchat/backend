use crate::database::*;

use super::hive::get_hive;
use futures::StreamExt;
use hive_pubsub::PubSub;
use mongodb::bson::doc;
use mongodb::options::FindOptions;

pub async fn generate_subscriptions(user: &User) -> Result<(), String> {
    let hive = get_hive();
    hive.subscribe(user.id.clone(), user.id.clone())?;

    if let Some(relations) = &user.relations {
        for relation in relations {
            hive.subscribe(user.id.clone(), relation.id.clone())?;
        }
    }

    let mut cursor = get_collection("channels")
        .find(
            doc! {
                "$or": [
                    {
                        "type": "SavedMessages",
                        "user": &user.id
                    },
                    {
                        "type": "DirectMessage",
                        "recipients": &user.id,
                        "active": true
                    },
                    {
                        "type": "Group",
                        "recipients": &user.id
                    }
                ]
            },
            FindOptions::builder().projection(doc! { "_id": 1 }).build(),
        )
        .await
        .map_err(|_| "Failed to fetch channels.".to_string())?;

    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            hive.subscribe(user.id.clone(), doc.get_str("_id").unwrap().to_string())?;
        }
    }

    Ok(())
}
