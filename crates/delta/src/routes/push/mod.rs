use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod subscribe;
mod unsubscribe;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![subscribe::subscribe, unsubscribe::unsubscribe]
}
