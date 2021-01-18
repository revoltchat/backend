use rocket::Route;

mod fetch_channel;
mod delete_channel;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_channel::req,
        delete_channel::req
    ]
}
