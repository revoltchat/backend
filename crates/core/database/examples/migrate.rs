use revolt_database::DatabaseInfo;

#[async_std::main]
async fn main() {
    let db = DatabaseInfo::Auto.connect().await.unwrap();
    db.migrate_database().await.unwrap();
}
