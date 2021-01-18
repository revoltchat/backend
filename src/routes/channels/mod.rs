use rocket::Route;

mod delete_channel;
mod fetch_channel;
mod message_fetch;
mod message_send;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_channel::req,
        delete_channel::req,
        message_send::req,
        message_fetch::req
    ]
}
