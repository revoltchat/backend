use rocket::Route;

mod fetch_channel;

pub fn routes() -> Vec<Route> {
    routes![
        fetch_channel::req
    ]
}
