pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::Rocket;

mod root;
mod users;
mod guild;
mod onboard;
mod channels;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket
        .mount("/", routes![root::root])
        .mount("/onboard", onboard::routes())
        .mount("/users", users::routes())
        .mount("/channels", channels::routes())
        .mount("/guild", guild::routes())
}
