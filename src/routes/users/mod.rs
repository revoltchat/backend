use rocket::Route;

mod add_friend;
mod block_user;
mod fetch_dms;
mod fetch_relationship;
mod fetch_relationships;
mod fetch_user;
mod find_mutual;
mod get_avatar;
mod get_default_avatar;
mod open_dm;
mod remove_friend;
mod unblock_user;

pub fn routes() -> Vec<Route> {
    routes![
        // User Information
        fetch_user::req,
        get_default_avatar::req,
        get_avatar::req,
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
