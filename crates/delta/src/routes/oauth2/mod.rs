use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod authorize_auth;
mod authorize_info;
mod authorized_bots;
mod revoke;
mod token;

// TODO
// Scopes:
// - identity
// - servers
// - full

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        authorize_auth::auth,
        authorize_info::info,
        authorized_bots::authorized_bots,
        revoke::revoke,
        token::token,
    ]
}
