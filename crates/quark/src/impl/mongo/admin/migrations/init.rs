use crate::r#impl::mongo::MongoDb;

use super::scripts::LATEST_REVISION;

use mongodb::bson::doc;
use mongodb::options::CreateCollectionOptions;

pub async fn create_database(db: &MongoDb) {
    info!("Creating database.");
    let db = db.db();

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

    db.create_collection("channel_unreads", None)
        .await
        .expect("Failed to create channel_unreads collection.");

    db.create_collection("migrations", None)
        .await
        .expect("Failed to create migrations collection.");

    db.create_collection("attachments", None)
        .await
        .expect("Failed to create attachments collection.");

    db.create_collection("user_settings", None)
        .await
        .expect("Failed to create user_settings collection.");

    db.create_collection("bots", None)
        .await
        .expect("Failed to create bots collection.");

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
                        "username": 1_i32
                    },
                    "name": "username",
                    "unique": true,
                    "collation": {
                        "locale": "en",
                        "strength": 2_i32
                    }
                }
            ]
        },
        None,
    )
    .await
    .expect("Failed to create username index.");

    db.run_command(
        doc! {
            "createIndexes": "messages",
            "indexes": [
                {
                    "key": {
                        "content": "text"
                    },
                    "name": "content"
                },
                {
                    "key": {
                        "channel": 1_i32
                    },
                    "name": "channel"
                },
                {
                    "key": {
                        "channel": 1_i32,
                        "_id": 1_i32
                    },
                    "name": "channel_id_compound"
                }
            ]
        },
        None,
    )
    .await
    .expect("Failed to create message index.");

    db.run_command(
        doc! {
            "createIndexes": "channel_unreads",
            "indexes": [
                {
                    "key": {
                        "_id.channel": 1_i32,
                        "_id.user": 1_i32,
                    },
                    "name": "compound_id"
                },
                {
                    "key": {
                        "_id.user": 1_i32,
                    },
                    "name": "user_id"
                }
            ]
        },
        None,
    )
    .await
    .expect("Failed to create channel_unreads index.");

    db.run_command(
        doc! {
            "createIndexes": "server_members",
            "indexes": [
                {
                    "key": {
                        "_id.server": 1_i32,
                        "_id.user": 1_i32,
                    },
                    "name": "compound_id"
                },
                {
                    "key": {
                        "_id.user": 1_i32,
                    },
                    "name": "user_id"
                }
            ]
        },
        None,
    )
    .await
    .expect("Failed to create server_members index.");

    db.collection("migrations")
        .insert_one(
            doc! {
                "_id": 0_i32,
                "revision": LATEST_REVISION
            },
            None,
        )
        .await
        .expect("Failed to save migration info.");

    info!("Created database.");
}
