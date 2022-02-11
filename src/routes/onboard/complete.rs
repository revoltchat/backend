use crate::util::regex::RE_USERNAME;
use revolt_quark::{models::User, Database, EmptyResponse, Error, Result};

use rauth::entities::Session;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
}

#[post("/complete", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Json<Data>,
) -> Result<EmptyResponse> {
    if user.is_some() {
        return Err(Error::AlreadyOnboarded);
    }

    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if db.is_username_taken(&data.username).await? {
        return Err(Error::UsernameTaken);
    }

    let user = User {
        id: session.user_id,
        username: data.username,
        ..Default::default()
    };

    db.insert_user(&user).await.map(|_| EmptyResponse)
}
