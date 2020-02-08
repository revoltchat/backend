use mongodb::{ Client, Collection, Database };
use std::env;

use once_cell::sync::OnceCell;
static DBCONN: OnceCell<Client> = OnceCell::new();

pub fn connect() {
	let client = Client::with_uri_str(
			&env::var("DB_URI").expect("DB_URI not in environment variables!"))
		.expect("Failed to init db connection.");

	DBCONN.set(client).unwrap();
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

pub mod user;
pub mod channel;
pub mod message;
