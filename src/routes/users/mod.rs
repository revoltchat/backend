use rocket::Route;

mod add_friend;
mod block_user;
mod change_username;
mod edit_user;
mod fetch_dms;
mod fetch_profile;
mod fetch_relationship;
mod fetch_relationships;
mod fetch_user;
mod find_mutual;
mod get_default_avatar;
mod open_dm;
mod remove_friend;
mod unblock_user;
mod fetch_self;

pub fn routes() -> Vec<Route> {
    routes![
        // User Information
        fetch_self::req,
        fetch_user::req,
        edit_user::req,
        change_username::req,
        get_default_avatar::req,
        fetch_profile::req,
        // Direct Messaging
        fetch_dms::req,
        open_dm::req,
        // Relationships
        find_mutual::req,
        fetch_relationships::req,
        fetch_relationship::req,
        add_friend::req,
        remove_friend::req,
        block_user::req,
        unblock_user::req,
    ]
}
