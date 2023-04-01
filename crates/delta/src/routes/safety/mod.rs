use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod edit_report;
mod fetch_report;
mod fetch_reports;
mod report_content;

mod fetch_snapshot;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        // Reports
        edit_report::edit_report,
        fetch_report::fetch_report,
        fetch_reports::fetch_reports,
        report_content::report_content,
        // Snapshots
        fetch_snapshot::fetch_snapshot
    ]
}
