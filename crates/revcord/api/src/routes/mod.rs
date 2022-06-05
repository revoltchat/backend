pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::{Build, Rocket};
use rocket_okapi::{settings::OpenApiSettings, okapi::openapi3::OpenApi};

pub mod channels;

pub fn mount(mut rocket: Rocket<Build>) -> Rocket<Build> {
    let settings = OpenApiSettings::default();

    mount_endpoints_and_merged_docs! {
        rocket, "/".to_owned(), settings,
        "/" => (vec![], custom_openapi_spec()),
        "/channels" => channels::routes(),
    };

    rocket
}


fn custom_openapi_spec() -> OpenApi {
    OpenApi {
        openapi: OpenApi::default_version(),
        ..Default::default()
    }
}
