use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod edit_report;
mod fetch_reports;
mod report_content;

mod fetch_snapshot;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        // Reports
        report_content::report_content,
        fetch_reports::fetch_reports,
        edit_report::edit_report,
        // Snapshots
        fetch_snapshot::fetch_snapshot
    ]
}
