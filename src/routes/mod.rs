use rocket::Rocket;

mod account;

pub fn mount(rocket: Rocket) -> Rocket {
	rocket
		.mount("/api/account", routes![ account::root, account::create, account::verify_email ])
}
