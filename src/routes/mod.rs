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
        "/users" => users::routes(),
        "/bots" => bots::routes(),
        "/channels" => channels::routes(),
        "/servers" => servers::routes(),
        "/invites" => invites::routes(),
        "/auth/account" => rauth::web::account::routes(),
        "/auth/session" => rauth::web::session::routes(),
        "/onboard" => onboard::routes(),
        "/push" => push::routes(),
        "/sync" => sync::routes(),
    };

    rocket
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
            description: Some(
                "User-first privacy focused chat platform.\n\n<!-- ReDoc-Inject: <security-definitions> -->".to_owned(),
            ),
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
        }, Server {
            url: "http://local.revolt.chat:8000".to_owned(),
            description: Some("Local Revolt Environment".to_owned()),
            ..Default::default()
        }],
        external_docs: Some(ExternalDocs {
            url: "https://developers.revolt.chat".to_owned(),
            description: Some("Revolt Developer Documentation".to_owned()),
            ..Default::default()
        }),
        extensions,
        /*tags: vec![
            Tag {
                name: "aaa".to_owned(),
                description: Some("aaa".to_owned()),
                ..Default::default()
            }
        ],*/
        ..Default::default()
    }
}
