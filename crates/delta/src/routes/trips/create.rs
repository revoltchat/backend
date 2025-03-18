use revolt_database::trips::model::Trip;
use revolt_database::{Database, DatabaseTrait};
use revolt_quark::models::User;
use revolt_result::Result;
use revolt_rocket_okapi::openapi;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{post, State};

/// Create a new trip
///
/// Creates a new trip using the authenticated user's ID.
#[openapi]
#[post("/create", format = "json", data = "<trip>")]
pub async fn create_trip(db: &State<Database>, user: User, mut trip: Json<Trip>) -> Result<Status> {
    trip.0.user_id = user.id;

    // Insert trip will handle marking other trips as deleted
    db.insert_trip(&trip.0).await?;

    Ok(Status::Created)
}
