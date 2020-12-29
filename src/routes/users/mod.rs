use rocket::Route;

mod fetch_user;
mod fetch_dms;
mod open_dm;
mod fetch_relationships;
mod fetch_relationship;
mod add_friend;
mod remove_friend;
mod block_user;
mod unblock_user;

pub fn routes() -> Vec<Route> {
    routes! [
        // User Information
        fetch_user::req,

        // Direct Messaging
        fetch_dms::req,
        open_dm::req,

        // Relationships
        fetch_relationships::req,
        fetch_relationship::req,
        add_friend::req,
        remove_friend::req,
        block_user::req,
        unblock_user::req,
    ]
}
