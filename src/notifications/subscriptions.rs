use crate::database::*;

use super::hive::get_hive;
use futures::StreamExt;
use hive_pubsub::PubSub;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::options::FindOptions;

pub async fn generate_subscriptions(user: &User) -> Result<(), String> {
    let hive = get_hive();
    hive.subscribe(user.id.clone(), user.id.clone())?;

    if let Some(relations) = &user.relations {
        for relation in relations {
            hive.subscribe(user.id.clone(), relation.id.clone())?;
        }
    }

    let server_ids = User::fetch_server_ids(&user.id)
        .await
        .map_err(|_| "Failed to fetch memberships.".to_string())?;

    let channel_ids = get_collection("servers")
        .find(
            doc! {
                "_id": {
                    "$in": &server_ids
                }
            },
            None,
        )
        .await
        .map_err(|_| "Failed to fetch servers.".to_string())?
        .filter_map(async move |s| s.ok())
        .collect::<Vec<Document>>()
        .await
        .into_iter()
        .filter_map(|x| {
            x.get_array("channels").ok().map(|v| {
                v.into_iter()
                    .filter_map(|x| x.as_str().map(|x| x.to_string()))
                    .collect::<Vec<String>>()
            })
        })
        .flatten()
        .collect::<Vec<String>>();

    for id in server_ids {
        hive.subscribe(user.id.clone(), id)?;
    }

    for id in channel_ids {
        hive.subscribe(user.id.clone(), id)?;
    }

    let mut cursor = get_collection("channels")
        .find(
            doc! {
                "$or": [
                    {
                        "channel_type": "SavedMessages",
                        "user": &user.id
                    },
                    {
                        "channel_type": "DirectMessage",
                        "recipients": &user.id
                    },
                    {
                        "channel_type": "Group",
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
