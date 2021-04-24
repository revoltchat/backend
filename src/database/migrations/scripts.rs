use crate::database::get_collection;

use log::info;
use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, from_document};

#[derive(Serialize, Deserialize)]
struct MigrationInfo {
    _id: i32,
    revision: i32,
}

pub const LATEST_REVISION: i32 = 1;

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
        info!("Running migration [revision 1 / 2021-04-24]: Migrate to Autumn v1.0.0.");

        let messages = get_collection("messages");
        let attachments = get_collection("attachments");

        messages.update_many(
            doc! { "attachment": { "$exists": 1 } },
            doc! { "$set": { "attachment.tag": "attachments", "attachment.size": 0 } },
            None
        )
        .await
        .expect("Failed to update messages.");

        attachments.update_many(
            doc! { },
            doc! { "$set": { "tag": "attachments", "size": 0 } },
            None
        )
        .await
        .expect("Failed to update attachments.");
    }

    // Reminder to update LATEST_REVISION when adding new migrations.
    LATEST_REVISION
}
