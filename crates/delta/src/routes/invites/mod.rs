use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod invite_delete;
mod invite_fetch;
mod invite_join;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![invite_fetch::req, invite_join::req, invite_delete::req]
}
