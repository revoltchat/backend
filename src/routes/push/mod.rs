use rocket::Route;

mod subscribe;
mod unsubscribe;

pub fn routes() -> Vec<Route> {
    routes![subscribe::req, unsubscribe::req]
}
