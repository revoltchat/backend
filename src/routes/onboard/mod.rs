use rocket::Route;

mod hello;
mod complete;

pub fn routes() -> Vec<Route> {
    routes! [
        hello::req,
        complete::req
    ]
}
