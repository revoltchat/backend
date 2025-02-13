pub mod create;
pub mod fetch;

use rocket::Route;
use revolt_rocket_okapi::openapi_get_routes_spec;
use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![create::create_trip, fetch::fetch_trips]
}
