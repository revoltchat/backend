use rocket::Route;

mod create;
mod invite;

pub fn routes() -> Vec<Route> {
    routes![
        create::create_bot,
        invite::invite_bot
    ]
}
