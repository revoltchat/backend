use rocket::Route;

mod server_create;
mod server_delete;
mod server_fetch;
mod server_edit;

mod channel_create;

mod members_fetch;

pub fn routes() -> Vec<Route> {
    routes![
        server_create::req,
        server_delete::req,
        server_fetch::req,
        server_edit::req,
        channel_create::req,
        members_fetch::req
    ]
}
