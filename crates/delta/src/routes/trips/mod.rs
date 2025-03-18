pub mod create;
pub mod delete;
pub mod fetch;
pub mod trip_comments;

use revolt_rocket_okapi::openapi_get_routes_spec;
use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        create::create_trip,
        fetch::fetch_trips,
        delete::delete_trip,
        trip_comments::create_trip_comment,
        trip_comments::fetch_trip_comments
    ]
}
