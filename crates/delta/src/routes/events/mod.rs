use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod event_create;
mod event_delete;
mod event_edit;
mod event_fetch;
mod event_list;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        event_create::create_event,
        event_delete::delete_event,
        event_edit::update_event,
        event_fetch::get_event,
        event_list::list_events,
    ]
}
