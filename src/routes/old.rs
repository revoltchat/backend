use crate::database;
use bson::{ bson, doc, ordered::OrderedDocument };

#[get("/")]
pub fn root() -> String {
	let client = database::get_connection();
	let cursor = client.database("revolt").collection("users").find(None, None).unwrap();

	let results: Vec<Result<OrderedDocument, mongodb::error::Error>> = cursor.collect();

	format!("ok boomer, users: {}", results.len())
}

#[get("/reg")]
pub fn reg() -> String {
	let client = database::get_connection();
	let col = client.database("revolt").collection("users");

	col.insert_one(doc! { "username": "test" }, None).unwrap();

	format!("inserted")
}
