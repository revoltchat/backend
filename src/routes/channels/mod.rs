use rocket::Route;

mod fetch_channel;
mod delete_channel;
mod message_send;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_channel::req,
        delete_channel::req,
        message_send::req
    ]
}
