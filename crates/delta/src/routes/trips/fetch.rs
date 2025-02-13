use rocket::{get, State};
use rocket::serde::json::Json;
use chrono::Utc;
use revolt_database::trips::model::Trip;
use revolt_database::{DatabaseTrait, Database};
use revolt_result::Result;
use revolt_rocket_okapi::openapi;

#[openapi]
#[get("/search?<date>&<destination>")]
pub async fn fetch_trips(
    db: &State<Database>,
    date: Option<String>,
    destination: String,
) -> Result<Json<Vec<Trip>>> {
    eprintln!("🚀 fetch_trips called with date={:?} destination={}", date, destination);

    // Parse date
    let parsed = date
        .and_then(|d| d.parse().ok())
        .unwrap_or_else(Utc::now);

    eprintln!("Parsed date = {}", parsed);

    let trips = db.fetch_trips_by_date_and_destination(parsed, &destination).await?;

    eprintln!("Found {} trips in route", trips.len());
    Ok(Json(trips))
}
