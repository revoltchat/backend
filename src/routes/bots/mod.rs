use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod create;
mod delete;
mod edit;
mod fetch;
mod fetch_owned;
mod fetch_public;
mod invite;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        create::create_bot,
        invite::invite_bot,
        fetch_public::fetch_public_bot,
        fetch::fetch_bot,
        fetch_owned::fetch_owned_bots,
        edit::edit_bot,
        delete::delete_bot,
    ]
}
