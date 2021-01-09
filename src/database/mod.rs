use crate::util::variables::MONGO_URI;

use mongodb::{Client, Collection, Database};
use once_cell::sync::OnceCell;

static DBCONN: OnceCell<Client> = OnceCell::new();

pub async fn connect() {
    let client = Client::with_uri_str(&MONGO_URI)
        .await
        .expect("Failed to init db connection.");

    DBCONN.set(client).unwrap();
    migrations::run_migrations().await;
}

pub fn get_connection() -> &'static Client {
    DBCONN.get().unwrap()
}

pub fn get_db() -> Database {
    get_connection().database("revolt")
}

pub fn get_collection(collection: &str) -> Collection {
    get_db().collection(collection)
}

pub mod entities;
pub mod guards;
pub mod migrations;
pub mod permissions;

pub use entities::*;
pub use guards::*;
pub use permissions::*;
