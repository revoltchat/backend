use rocket::Route;

mod hello;

pub fn routes() -> Vec<Route> {
    routes! [
        hello::req
    ]
}
