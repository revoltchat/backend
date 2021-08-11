use rocket::Route;

mod create;

pub fn routes() -> Vec<Route> {
    routes![
        create::create_bot
    ]
}
