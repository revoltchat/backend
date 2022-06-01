use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod complete;
mod hello;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![hello::req, complete::req]
}
