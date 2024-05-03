use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod get_settings;
mod get_unreads;
mod set_settings;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![get_settings::req, set_settings::req, get_unreads::req]
}
