use super::super::get_db;
use super::scripts::LATEST_REVISION;

use log::info;
use mongodb::bson::doc;
use mongodb::options::CreateCollectionOptions;

pub fn create_database() {
    info!("Creating database.");
    let db = get_db();

    db.create_collection("users", None)
        .expect("Failed to create users collection.");
    db.create_collection("channels", None)
        .expect("Failed to create channels collection.");
    db.create_collection("guilds", None)
        .expect("Failed to create guilds collection.");
    db.create_collection("members", None)
        .expect("Failed to create members collection.");
    db.create_collection("messages", None)
        .expect("Failed to create messages collection.");
    db.create_collection("migrations", None)
        .expect("Failed to create migrations collection.");

    db.create_collection(
        "pubsub",
        CreateCollectionOptions::builder()
            .capped(true)
            .size(1_000_000)
            .build(),
    )
    .expect("Failed to create pubsub collection.");

    db.collection("migrations")
        .insert_one(
            doc! {
                "_id": 0,
                "revision": LATEST_REVISION
            },
            None,
        )
        .expect("Failed to save migration info.");

    info!("Created database.");
}
