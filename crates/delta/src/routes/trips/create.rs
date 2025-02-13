use rocket::{post, State};
use rocket::serde::json::Json;
use revolt_database::trips::model::Trip;
use revolt_database::{DatabaseTrait, Database};
use revolt_result::Result;
use revolt_rocket_okapi::openapi;

#[openapi]
#[post("/create", format = "json", data = "<trip>")]
pub async fn create_trip(
    db: &State<Database>,
    trip: Json<Trip>,
) -> Result<Json<Trip>> {
    db.insert_trip(&trip.0).await?;
    Ok(trip)
}
