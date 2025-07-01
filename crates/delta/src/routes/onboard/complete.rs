use authifier::models::Session;
use revolt_database::{Database, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};

use rocket::State;
use crate::util::json::{Json, Validate};


/// # Complete Onboarding
///
/// This sets a new username, completes onboarding and allows a user to start using Revolt.
#[openapi(tag = "Onboarding")]
#[post("/complete", data = "<data>")]
pub async fn complete(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Validate<Json<v0::DataOnboard>>,
) -> Result<Json<v0::User>> {
    if user.is_some() {
        return Err(create_error!(AlreadyOnboarded));
    }

    let data = data.into_inner().into_inner();

    Ok(Json(
        User::create(db, data.username, session.user_id, None)
            .await?
            .into_self(false)
            .await,
    ))
}
