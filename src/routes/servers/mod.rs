use rocket::Route;

mod server_create;
mod server_delete;

pub fn routes() -> Vec<Route> {
    routes![server_create::req, server_delete::req]
}
