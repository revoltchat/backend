use revolt_database::mongodb::bson::oid::ObjectId;
use revolt_database::trips::model::TripComment;
use revolt_database::{Database, DatabaseTrait};
use revolt_quark::models::User;
use revolt_result::Result;
use revolt_rocket_okapi::openapi;
use revolt_rocket_okapi::revolt_okapi::schemars::JsonSchema;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, JsonSchema)]
pub struct CreateTripCommentRequest {
    #[schemars(with = "String")]
    pub trip_id: ObjectId,
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FetchTripCommentsRequest {
    #[schemars(with = "String")]
    pub trip_id: ObjectId,
    pub destination: String,
}

#[derive(Serialize, JsonSchema)]
pub struct FetchTripCommentsResponse {
    pub comments: Vec<TripComment>,
}

/// Create a comment on a trip
///
/// Creates a new comment on the specified trip using the authenticated user's ID.
#[openapi]
#[post("/comment", format = "json", data = "<request>")]
pub async fn create_trip_comment(
    db: &State<Database>,
    user: User,
    request: Json<CreateTripCommentRequest>,
) -> Result<Status> {
    let comment = TripComment {
        id: None,
        trip_id: request.trip_id,
        user_id: user.id,
        content: request.content.clone(),
        created_at: chrono::Utc::now(),
    };

    db.create_trip_comment(&comment).await?;
    Ok(Status::Created)
}

/// Fetch comments for a trip in a destination
///
/// Returns all comments for the specified trip in the destination, sorted by date (newest first)
#[openapi]
#[post("/comments/fetch", format = "json", data = "<request>")]
pub async fn fetch_trip_comments(
    db: &State<Database>,
    _user: User, // Authentication required but user not needed
    request: Json<FetchTripCommentsRequest>,
) -> Result<Json<FetchTripCommentsResponse>> {
    let comments = db
        .fetch_trip_comments_by_destination(request.trip_id, &request.destination)
        .await?;

    Ok(Json(FetchTripCommentsResponse { comments }))
}
