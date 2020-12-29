use rocket::Route;

mod fetch_user;
mod fetch_dms;

pub fn routes() -> Vec<Route> {
    routes! [
        fetch_user::req,
        fetch_dms::req
    ]
}
