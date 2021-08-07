use rocket::Route;

mod server_create;
mod server_delete;
mod server_edit;
mod server_fetch;
mod server_ack;

mod channel_create;

mod member_edit;
mod member_fetch;
mod member_fetch_all;
mod member_remove;

mod ban_create;
mod ban_list;
mod ban_remove;

mod invites_fetch;

mod roles_create;
mod roles_edit;
mod roles_delete;
mod permissions_set;
mod permissions_set_default;

pub fn routes() -> Vec<Route> {
    routes![
        server_create::req,
        server_delete::req,
        server_fetch::req,
        server_edit::req,
        server_ack::req,
        channel_create::req,
        member_fetch_all::req,
        member_remove::req,
        member_fetch::req,
        member_edit::req,
        ban_create::req,
        ban_remove::req,
        ban_list::req,
        invites_fetch::req,
        roles_create::req,
        roles_edit::req,
        roles_delete::req,
        permissions_set::req,
        permissions_set_default::req
    ]
}
