#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

pub mod websocket;
pub mod database;
pub mod guards;
pub mod routes;
pub mod email;

use dotenv;
use std::thread;

fn main() {
	dotenv::dotenv().ok();
	database::connect();

	thread::spawn(|| {
		websocket::launch_server();
	});

	routes::mount(rocket::ignite()).launch();
}
