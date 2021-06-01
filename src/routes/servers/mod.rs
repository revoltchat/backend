use rocket::Route;

mod server_create;
mod server_delete;
mod server_edit;

pub fn routes() -> Vec<Route> {
    routes![server_create::req, server_delete::req, server_edit::req]
}
