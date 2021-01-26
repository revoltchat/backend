use super::super::{get_collection, get_db};

use crate::rocket::futures::StreamExt;
use log::info;
use mongodb::bson::{doc, from_document};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MigrationInfo {
    _id: i32,
    revision: i32,
}

pub const LATEST_REVISION: i32 = 0;

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

    // Reminder to update LATEST_REVISION when adding new migrations.
    LATEST_REVISION
}
