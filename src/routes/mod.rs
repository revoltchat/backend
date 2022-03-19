pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::{Build, Rocket};
use rocket_okapi::settings::OpenApiSettings;

mod bots;
mod channels;
mod invites;
mod onboard;
mod push;
mod root;
mod servers;
mod sync;
mod users;

pub fn mount(mut rocket: Rocket<Build>) -> Rocket<Build> {
    let settings = OpenApiSettings::default();

    mount_endpoints_and_merged_docs! {
        rocket, "/".to_owned(), settings,
        "" => openapi_get_routes_spec![root::root, root::ping],
        "/bots" => bots::routes(),
        "/channels" => channels::routes(),
    };

    rocket
        .mount("/auth/account", rauth::web::account::routes())
        .mount("/auth/session", rauth::web::session::routes())
        .mount("/onboard", onboard::routes())
        .mount("/users", users::routes())
        .mount("/servers", servers::routes())
        .mount("/invites", invites::routes())
        .mount("/push", push::routes())
        .mount("/sync", sync::routes())
}
