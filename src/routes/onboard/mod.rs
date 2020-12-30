use rocket::Route;

mod complete;
mod hello;

pub fn routes() -> Vec<Route> {
    routes![hello::req, complete::req]
}
