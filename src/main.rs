#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

pub mod database;
pub mod guards;
pub mod routes;
pub mod email;

use dotenv;

fn main() {
	dotenv::dotenv().ok();
	database::connect();

	routes::mount(rocket::ignite()).launch();
}
