use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod accounts;
mod cases;
mod meta;
mod reports;
mod roles;
mod search;
mod users;

mod object_edit_note;
mod object_get_note;
mod util;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        object_edit_note::admin_object_edit_note,
        object_get_note::admin_object_get_note,
        meta::create_user::admin_create_user,
        meta::edit_user::admin_edit_user,
        meta::fetch_users::admin_fetch_users,
        meta::create_token::admin_create_token,
        meta::revoke_token::admin_revoke_token
    ]
}
