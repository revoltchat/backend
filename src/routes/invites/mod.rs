use rocket::Route;

mod invite_delete;
mod invite_fetch;
mod invite_join;

pub fn routes() -> Vec<Route> {
    routes![invite_fetch::req, invite_join::req, invite_delete::req]
}
