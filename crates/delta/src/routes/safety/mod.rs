use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;

mod report_content;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        // Reports
        report_content::report_content
    ]
}
