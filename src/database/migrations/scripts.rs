use super::super::{get_collection, get_db};

use crate::rocket::futures::StreamExt;
use log::info;
use mongodb::bson::{doc, from_bson, from_document, Bson};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MigrationInfo {
    _id: i32,
    revision: i32,
}

pub const LATEST_REVISION: i32 = 3;

pub async fn migrate_database() {
    let migrations = get_collection("migrations");
    let data = migrations
        .find_one(None, None)
        .await
        .expect("Failed to fetch migration data.");

    if let Some(doc) = data {
        let info: MigrationInfo =
            from_document(doc).expect("Failed to read migration information.");

        let revision = run_migrations(info.revision).await;

        migrations
            .update_one(
                doc! {
                    "_id": info._id
                },
                doc! {
                    "$set": {
                        "revision": revision
                    }
                },
                None,
            )
            .await
            .expect("Failed to commit migration information.");

        info!("Migration complete. Currently at revision {}.", revision);
    } else {
        panic!("Database was configured incorrectly, possibly because initalization failed.")
    }
}

pub async fn run_migrations(revision: i32) -> i32 {
    info!("Starting database migration.");

    if revision <= 0 {
        info!("Running migration [revision 0]: Test migration system.");
    }

    if revision <= 1 {
        info!("Running migration [revision 1]: Add channels to guild object.");

        let col = get_collection("guilds");
        let mut guilds = col
            .find(
                None,
                FindOptions::builder().projection(doc! { "_id": 1 }).build(),
            )
            .await
            .expect("Failed to fetch guilds.");

        let mut result = get_collection("channels")
            .find(
                doc! {
                    "type": 2
                },
                FindOptions::builder()
                    .projection(doc! { "_id": 1, "guild": 1 })
                    .build(),
            )
            .await
            .expect("Failed to fetch channels.");

        let mut channels = vec![];
        while let Some(doc) = result.next().await {
            let channel = doc.expect("Failed to fetch channel.");
            let id = channel
                .get_str("_id")
                .expect("Failed to get channel id.")
                .to_string();

            let gid = channel
                .get_str("guild")
                .expect("Failed to get guild id.")
                .to_string();

            channels.push((id, gid));
        }

        while let Some(doc) = guilds.next().await {
            let guild = doc.expect("Failed to fetch guild.");
            let id = guild.get_str("_id").expect("Failed to get guild id.");

            let list: Vec<String> = channels
                .iter()
                .filter(|x| x.1 == id)
                .map(|x| x.0.clone())
                .collect();

            col.update_one(
                doc! {
                    "_id": id
                },
                doc! {
                    "$set": {
                        "channels": list
                    }
                },
                None,
            )
            .await
            .expect("Failed to update guild.");
        }
    }

    if revision <= 2 {
        info!("Running migration [revision 2]: Add username index to users.");

        get_db()
            .run_command(
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
    }

    // Reminder to update LATEST_REVISION when adding new migrations.
    LATEST_REVISION
}
