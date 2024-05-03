use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod invite_delete;
mod invite_fetch;
mod invite_join;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![invite_fetch::req, invite_join::req, invite_delete::req]
}
