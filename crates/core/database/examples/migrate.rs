use revolt_database::DatabaseInfo;
use revolt_models::*;

#[async_std::main]
async fn main() {
    let db = Database(DatabaseInfo::Auto.connect().await.unwrap());
    db.migrate_database().await.unwrap();
}
