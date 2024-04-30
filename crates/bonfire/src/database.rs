use once_cell::sync::OnceCell;
use revolt_quark::{Database, DatabaseInfo};

static DBCONN: OnceCell<Database> = OnceCell::new();

/// Connect Bonfire to the database.
pub async fn connect() {
    let database = DatabaseInfo::Auto
        .connect()
        .await
        .expect("Failed to connect to the database.");

    DBCONN.set(database).expect("Setting `Database`");
}

/// Get a reference to the current database.
pub fn get_db() -> &'static Database {
    DBCONN.get().expect("Valid `Database`")
}
