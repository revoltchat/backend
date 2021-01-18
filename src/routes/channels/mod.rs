use rocket::Route;

mod fetch_channel;
mod delete_channel;
mod message_send;
mod message_fetch;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_channel::req,
        delete_channel::req,
        message_send::req,
        message_fetch::req
    ]
}
