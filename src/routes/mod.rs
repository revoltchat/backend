pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::{Build, Rocket};

mod bots;
mod channels;
mod invites;
mod onboard;
mod push;
mod root;
mod servers;
mod sync;
mod users;

pub fn mount(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .mount("/", routes![root::root, root::ping])
        .mount("/onboard", onboard::routes())
        .mount("/users", users::routes())
        .mount("/channels", channels::routes())
        .mount("/servers", servers::routes())
        .mount("/bots", bots::routes())
        .mount("/invites", invites::routes())
        .mount("/push", push::routes())
        .mount("/sync", sync::routes())
}
