use rocket::Route;

mod server_create;
mod server_delete;
mod server_fetch;
mod server_edit;

mod channel_create;

mod member_fetch_all;
mod member_remove;
mod member_fetch;
mod member_edit;

mod ban_create;
mod ban_remove;
mod ban_list;

mod invites_fetch;

pub fn routes() -> Vec<Route> {
    routes![
        server_create::req,
        server_delete::req,
        server_fetch::req,
        server_edit::req,
        channel_create::req,
        member_fetch_all::req,
        member_remove::req,
        member_fetch::req,
        member_edit::req,
        ban_create::req,
        ban_remove::req,
        ban_list::req,
        invites_fetch::req
    ]
}
