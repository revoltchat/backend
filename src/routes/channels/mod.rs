use rocket::Route;

mod delete_channel;
mod edit_channel;
mod fetch_channel;
mod fetch_members;
mod group_add_member;
mod group_create;
mod group_remove_member;
mod invite_channel;
mod join_call;
mod message_delete;
mod message_edit;
mod message_fetch;
mod message_query;
mod message_query_stale;
mod message_send;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_channel::req,
        fetch_members::req,
        delete_channel::req,
        edit_channel::req,
        invite_channel::req,
        message_send::req,
        message_query::req,
        message_query_stale::req,
        message_fetch::req,
        message_edit::req,
        message_delete::req,
        group_create::req,
        group_add_member::req,
        group_remove_member::req,
        join_call::req,
    ]
}
