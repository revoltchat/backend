use rocket::Route;

mod get_settings;
mod set_settings;

pub fn routes() -> Vec<Route> {
    routes![
        get_settings::req,
        set_settings::req
    ]
}
