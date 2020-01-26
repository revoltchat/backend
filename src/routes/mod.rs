use rocket::Rocket;

mod account;
mod user;

pub fn mount(rocket: Rocket) -> Rocket {
	rocket
		.mount("/api/account", routes![ account::create, account::verify_email, account::resend_email, account::login ])
		.mount("/api/users", routes![ user::me, user::dms, user::lookup ])
}
