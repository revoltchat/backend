use rocket::Route;

mod fetch_user;

pub fn routes() -> Vec<Route> {
    routes! [
        fetch_user::req
    ]
}
