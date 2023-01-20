use rocket::Route;
use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;

mod webhook_delete;
mod webhook_edit;
mod webhook_execute;
mod webhook_fetch;
mod webhook_execute_github;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        webhook_delete::req,
        webhook_edit::req,
        webhook_execute::req,
        webhook_fetch::req,
        webhook_execute_github::req
    ]
}
