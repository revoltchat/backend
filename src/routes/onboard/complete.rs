use crate::database::*;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use mongodb::options::{Collation, FindOneOptions};
use rauth::auth::Session;
use regex::Regex;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9_.]+$").unwrap();
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
}

#[post("/complete", data = "<data>")]
pub async fn req(session: Session, user: Option<User>, data: Json<Data>) -> Result<()> {
    if user.is_some() {
        Err(Error::AlreadyOnboarded)?
    }

    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if data.username == "revolt" {
        Err(Error::UsernameTaken)?
    }

    let col = get_collection("users");
    if col
        .find_one(
            doc! {
                "username": &data.username
            },
            FindOneOptions::builder()
                .collation(Collation::builder().locale("en").strength(2).build())
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find_one",
            with: "user",
        })?
        .is_some()
    {
        Err(Error::UsernameTaken)?
    }

    col.insert_one(
        doc! {
            "_id": session.user_id,
            "username": &data.username
        },
        None,
    )
    .await
    .map_err(|_| Error::DatabaseError {
        operation: "insert_one",
        with: "user",
    })?;

    Ok(())
}
