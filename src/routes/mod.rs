pub use rocket::response::Redirect;
pub use rocket::http::Status;
use rocket::{Phase, Rocket};

mod channels;
mod invites;
mod onboard;
mod push;
mod root;
mod servers;
mod sync;
mod users;

pub fn mount<T: Phase>(rocket: Rocket<T>) -> Rocket<T> {
    rocket
        .mount("/", routes![root::root])
        .mount("/onboard", onboard::routes())
        .mount("/users", users::routes())
        .mount("/channels", channels::routes())
        .mount("/servers", servers::routes())
        .mount("/invites", invites::routes())
        .mount("/push", push::routes())
        .mount("/sync", sync::routes())
}
