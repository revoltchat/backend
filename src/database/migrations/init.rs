use super::super::get_db;
use super::scripts::LATEST_REVISION;

use log::info;
use mongodb::bson::doc;
use mongodb::options::CreateCollectionOptions;

pub async fn create_database() {
    info!("Creating database.");
    let db = get_db();

    db.create_collection("accounts", None)
        .await
        .expect("Failed to create accounts collection.");

    db.create_collection("users", None)
        .await
        .expect("Failed to create users collection.");

    db.create_collection("channels", None)
        .await
        .expect("Failed to create channels collection.");

    db.create_collection("messages", None)
        .await
        .expect("Failed to create messages collection.");

    db.create_collection("servers", None)
        .await
        .expect("Failed to create servers collection.");

    db.create_collection("server_members", None)
        .await
        .expect("Failed to create server_members collection.");

    db.create_collection("server_bans", None)
        .await
        .expect("Failed to create server_bans collection.");

    db.create_collection("channel_invites", None)
        .await
        .expect("Failed to create channel_invites collection.");

    db.create_collection("migrations", None)
        .await
        .expect("Failed to create migrations collection.");

    db.create_collection("attachments", None)
        .await
        .expect("Failed to create attachments collection.");

    db.create_collection("channel_unreads", None)
        .await
        .expect("Failed to create channel_unreads collection.");

    db.create_collection("user_settings", None)
        .await
        .expect("Failed to create user_settings collection.");

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
            "createIndexes": "accounts",
            "indexes": [
                {
                    "key": {
                        "email": 1
                    },
                    "name": "email",
                    "unique": true,
                    "collation": {
                        "locale": "en",
                        "strength": 2
                    }
                },
                {
                    "key": {
                        "email_normalised": 1
                    },
                    "name": "email_normalised",
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
    .expect("Failed to create account index.");

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
