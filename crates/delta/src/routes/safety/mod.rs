use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod fetch_reports;
mod report_content;

mod fetch_snapshot;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        // Reports
        report_content::report_content,
        fetch_reports::fetch_reports,
        // Snapshots
        fetch_snapshot::fetch_snapshot
    ]
}
