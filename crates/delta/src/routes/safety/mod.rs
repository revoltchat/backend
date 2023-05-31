use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod edit_report;
mod fetch_report;
mod fetch_reports;
mod report_content;

mod fetch_snapshots;

mod delete_strike;
mod edit_strike;
mod fetch_strikes;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        // Reports
        edit_report::edit_report,
        fetch_report::fetch_report,
        fetch_reports::fetch_reports,
        report_content::report_content,
        // Snapshots
        fetch_snapshots::fetch_snapshots,
        // Strikes
        fetch_strikes::fetch_strikes,
        edit_strike::edit_strike,
        delete_strike::delete_strike
    ]
}
