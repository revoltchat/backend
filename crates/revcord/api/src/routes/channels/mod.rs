use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod messages;

mod fetch_channel;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        fetch_channel::req,
        messages::message_send::req
    ]
}
