use crate::database::*;
use crate::notifications::events::ClientboundNotification;
use crate::util::result::{Error, Result};

use mongodb::bson::doc;
use rauth::auth::{Auth, Session};
use regex::Regex;
use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

// ! FIXME: should be global somewhere; maybe use config(?)
lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9_.]+$").unwrap();
}

#[derive(Validate, Serialize, Deserialize)]
pub struct Data {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: Option<String>,
    #[validate(length(min = 8, max = 72))]
    password: String,
}

#[patch("/<_ignore_id>/username", data = "<data>")]
pub async fn req(
    auth: State<'_, Auth>,
    session: Session,
    user: User,
    data: Json<Data>,
    _ignore_id: String,
) -> Result<()> {
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    auth.verify_password(&session, data.password.clone())
        .await
        .map_err(|_| Error::InvalidCredentials)?;

    let mut set = doc! {};
    if let Some(username) = &data.username {
        if User::is_username_taken(&username).await? {
            return Err(Error::UsernameTaken);
        }

        set.insert("username", username.clone());
    }

    get_collection("users")
        .update_one(doc! { "_id": &user.id }, doc! { "$set": set }, None)
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "user",
        })?;

    ClientboundNotification::UserUpdate {
        id: user.id.clone(),
        data: json!({
            "username": data.username
        }),
        clear: None,
    }
    .publish(user.id.clone());

    Ok(())
}
