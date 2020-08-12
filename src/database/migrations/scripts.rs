use super::super::get_collection;

use serde::{Serialize, Deserialize};
use mongodb::bson::{Bson, from_bson, doc};
use mongodb::options::FindOptions;
use log::info;

#[derive(Serialize, Deserialize)]
struct MigrationInfo {
    _id: i32,
    revision: i32
}

pub const LATEST_REVISION: i32 = 2;

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

    if revision <= 1 {
        info!("Running migration [revision 1]: Add channels to guild object.");

        let col = get_collection("guilds");
        let guilds = col.find(
            None,
            FindOptions::builder()
                .projection(doc! { "_id": 1 })
                .build()
        )
            .expect("Failed to fetch guilds.");
        
        let result = get_collection("channels").find(
            doc! {
                "type": 2
            },
            FindOptions::builder()
                .projection(doc! { "_id": 1, "guild": 1 })
                .build()
        ).expect("Failed to fetch channels.");

        let mut channels = vec![];
        for doc in result {
            let channel = doc.expect("Failed to fetch channel.");
            let id  = channel.get_str("_id").expect("Failed to get channel id.").to_string();
            let gid = channel.get_str("guild").expect("Failed to get guild id.").to_string();

            channels.push(( id, gid ));
        }
        
        for doc in guilds {
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
                None
            ).expect("Failed to update guild.");
        }
    }

    // Reminder to update LATEST_REVISION when adding new migrations.
    LATEST_REVISION
}
