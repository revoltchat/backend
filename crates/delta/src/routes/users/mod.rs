use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod add_friend;
mod block_user;
mod change_username;
mod edit_user;
mod fetch_dms;
mod fetch_profile;
mod fetch_self;
mod fetch_user;
mod fetch_user_flags;
mod find_mutual;
mod get_default_avatar;
mod open_dm;
mod remove_friend;
mod send_friend_request;
mod unblock_user;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        // User Information
        fetch_self::req,
        fetch_user::req,
        fetch_user_flags::fetch_user_flags,
        edit_user::req,
        change_username::req,
        get_default_avatar::req,
        fetch_profile::req,
        // Direct Messaging
        fetch_dms::req,
        open_dm::req,
        // Relationships
        find_mutual::req,
        add_friend::req,
        remove_friend::req,
        block_user::req,
        unblock_user::req,
        send_friend_request::req,
    ]
}
