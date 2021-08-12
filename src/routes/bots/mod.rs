use rocket::Route;

mod create;
mod invite;
mod fetch_public;
mod fetch;
mod fetch_owned;
mod edit;
mod delete;

pub fn routes() -> Vec<Route> {
    routes![
        create::create_bot,
        invite::invite_bot,
        fetch_public::fetch_public_bot,
        fetch::fetch_bot,
        fetch_owned::fetch_owned_bots,
        edit::edit_bot,
        delete::delete_bot,
    ]
}
