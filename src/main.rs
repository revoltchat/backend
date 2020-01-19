#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

pub mod database;
pub mod routes;

use dotenv;

fn main() {
	dotenv::dotenv().ok();
	database::connect();

	routes::mount(rocket::ignite()).launch();
}
