use revolt_database::mongodb::bson::{self, oid::ObjectId};
use revolt_database::{Database, DatabaseTrait};
use revolt_quark::models::User;
use revolt_result::Result;
use revolt_rocket_okapi::openapi;
use revolt_rocket_okapi::revolt_okapi::schemars;
use revolt_rocket_okapi::revolt_okapi::schemars::JsonSchema;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, State};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct DeleteTripRequest {
    #[serde(rename = "_id")]
    #[schemars(with = "String")]
    pub id: ObjectId,
}

/// Delete a trip
///
/// Marks a trip as deleted. Only the trip owner can delete their own trip.
#[openapi]
#[delete("/delete", format = "json", data = "<request>")]
pub async fn delete_trip(
    db: &State<Database>,
    user: User,
    request: Json<DeleteTripRequest>,
) -> Result<Status> {
    db.delete_trip(request.0.id, &user.id).await?;
    Ok(Status::Ok)
}
