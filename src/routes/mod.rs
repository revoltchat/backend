use rocket::Rocket;

mod account;

pub fn mount(rocket: Rocket) -> Rocket {
	rocket
		.mount("/api/v1", routes![account::root, account::reg])
}
