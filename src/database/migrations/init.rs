use super::super::get_db;
use super::scripts::LATEST_REVISION;

use log::info;
use mongodb::bson::doc;
use mongodb::options::CreateCollectionOptions;

pub async fn create_database() {
    info!("Creating database.");
    let db = get_db();

    db.create_collection("users", None)
        .await
        .expect("Failed to create users collection.");

    db.create_collection("channels", None)
        .await
        .expect("Failed to create channels collection.");

    db.create_collection("guilds", None)
        .await
        .expect("Failed to create guilds collection.");

    db.create_collection("members", None)
        .await
        .expect("Failed to create members collection.");

    db.create_collection("messages", None)
        .await
        .expect("Failed to create messages collection.");

    db.create_collection("migrations", None)
        .await
        .expect("Failed to create migrations collection.");

    db.create_collection(
        "pubsub",
        CreateCollectionOptions::builder()
            .capped(true)
            .size(1_000_000)
            .build(),
    )
    .await
    .expect("Failed to create pubsub collection.");

    db.run_command(
        doc! {
            "createIndexes": "users",
            "indexes": [
                {
                    "key": {
                        "username": 1
                    },
                    "name": "username",
                    "unique": true,
                    "collation": {
                        "locale": "en",
                        "strength": 2
                    }
                }
            ]
        },
        None,
    )
    .await
    .expect("Failed to create username index.");

    db.collection("migrations")
        .insert_one(
            doc! {
                "_id": 0,
                "revision": LATEST_REVISION
            },
            None,
        )
        .await
        .expect("Failed to save migration info.");

    info!("Created database.");
}
