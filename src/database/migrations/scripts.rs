use super::super::get_collection;

use serde::{Serialize, Deserialize};
use mongodb::bson::{Bson, from_bson, doc};
use log::info;

#[derive(Serialize, Deserialize)]
struct MigrationInfo {
    _id: i32,
    revision: i32
}

pub const LATEST_REVISION: i32 = 1;

pub fn migrate_database() {
    let migrations = get_collection("migrations");
    let data = migrations.find_one(None, None)
        .expect("Failed to fetch migration data.");
    
    if let Some(doc) = data {
        let info: MigrationInfo = from_bson(Bson::Document(doc))
            .expect("Failed to read migration information.");
        
        let revision = run_migrations(info.revision);

        migrations.update_one(
            doc! {
                "_id": info._id
            },
            doc! {
                "$set": {
                    "revision": revision
                }
            },
            None
        ).expect("Failed to commit migration information.");

        info!("Migration complete. Currently at revision {}.", revision);
    } else {
        panic!("Database was configured incorrectly, possibly because initalization failed.")
    }
}

pub fn run_migrations(revision: i32) -> i32 {
    info!("Starting database migration.");

    if revision <= 0 {
        info!("Running migration [revision 0]: Test migration system.");
    }

    // Reminder to update LATEST_REVISION when adding new migrations.
    LATEST_REVISION
}
