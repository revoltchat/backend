pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::{Build, Rocket};
use rocket_okapi::{okapi::openapi3::OpenApi, settings::OpenApiSettings};

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
        "/" => (vec![], custom_openapi_spec()),
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

fn custom_openapi_spec() -> OpenApi {
    use rocket_okapi::okapi::openapi3::*;

    let mut extensions = schemars::Map::new();
    extensions.insert(
        "x-logo".to_owned(),
        json!({
            "url": "https://revolt.chat/header.png",
            "altText": "Revolt Header"
        }),
    );

    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "Revolt API".to_owned(),
            description: Some("User-first privacy focused chat platform.".to_owned()),
            terms_of_service: Some("https://revolt.chat/terms".to_owned()),
            contact: Some(Contact {
                name: Some("Revolt Support".to_owned()),
                url: Some("https://revolt.chat".to_owned()),
                email: Some("contact@revolt.chat".to_owned()),
                ..Default::default()
            }),
            license: Some(License {
                name: "AGPLv3".to_owned(),
                url: Some("https://github.com/revoltchat/delta/blob/master/LICENSE".to_owned()),
                ..Default::default()
            }),
            version: "0.5.3-rc.1".to_owned(),
            ..Default::default()
        },
        servers: vec![Server {
            url: "https://api.revolt.chat".to_owned(),
            description: Some("Revolt API".to_owned()),
            ..Default::default()
        }],
        extensions,
        ..Default::default()
    }
}
