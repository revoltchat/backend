use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod subscribe;
mod unsubscribe;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![subscribe::req, unsubscribe::req]
}
