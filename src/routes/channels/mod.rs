use rocket::Route;

mod delete_channel;
mod fetch_channel;
mod message_delete;
mod message_edit;
mod message_fetch;
mod message_query;
mod message_send;
mod group_create;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_channel::req,
        delete_channel::req,
        message_send::req,
        message_query::req,
        message_fetch::req,
        message_edit::req,
        message_delete::req,
        group_create::req
    ]
}
