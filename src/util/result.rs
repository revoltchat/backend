use rocket::response::{self, Responder, Response};
use rocket::http::{ContentType, Status};
use validator::ValidationErrors;
use rocket::request::Request;
use serde::Serialize;
use std::io::Cursor;
use snafu::Snafu;
use json;

#[derive(Serialize, Debug, Snafu)]
#[serde(tag = "type")]
pub enum Error {
    #[snafu(display("This error has not been labelled."))]
    #[serde(rename = "unlabelled_error")]
    LabelMe,

    // ? Onboarding related errors.
    #[snafu(display("Already finished onboarding."))]
    #[serde(rename = "already_onboarded")]
    AlreadyOnboarded,
    
    // ? User related errors.
    #[snafu(display("Username has already been taken."))]
    #[serde(rename = "username_taken")]
    UsernameTaken,

    // ? General errors.
    #[snafu(display("Failed to validate fields."))]
    #[serde(rename = "failed_validation")]
    FailedValidation { error: ValidationErrors },
    #[snafu(display("Encountered a database error."))]
    #[serde(rename = "database_error")]
    DatabaseError { operation: &'static str, with: &'static str },

    /* #[snafu(display("Failed to validate fields."))]
    #[serde(rename = "failed_validation")]
    FailedValidation { error: ValidationErrors },
    #[snafu(display("Encountered a database error."))]
    #[serde(rename = "database_error")]
    DatabaseError,
    #[snafu(display("Encountered an internal error."))]
    #[serde(rename = "internal_error")]
    InternalError,
    #[snafu(display("Operation did not succeed."))]
    #[serde(rename = "operation_failed")]
    OperationFailed,
    #[snafu(display("Missing authentication headers."))]
    #[serde(rename = "missing_headers")]
    MissingHeaders,
    #[snafu(display("Invalid session information."))]
    #[serde(rename = "invalid_session")]
    InvalidSession,
    #[snafu(display("User account has not been verified."))]
    #[serde(rename = "unverified_account")]
    UnverifiedAccount,
    #[snafu(display("This user does not exist!"))]
    #[serde(rename = "unknown_user")]
    UnknownUser,
    #[snafu(display("Email is use."))]
    #[serde(rename = "email_in_use")]
    EmailInUse,
    #[snafu(display("Wrong password."))]
    #[serde(rename = "wrong_password")]
    WrongPassword, */
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// HTTP response builder for Error enum
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self {
            Error::AlreadyOnboarded => Status::Forbidden,
            Error::DatabaseError { .. } => Status::InternalServerError,
            Error::FailedValidation { .. } => Status::UnprocessableEntity,
            Error::LabelMe => Status::InternalServerError,
            Error::UsernameTaken => Status::Conflict,
        };

        // Serialize the error data structure into JSON.
        let string = json!(self).to_string();

        // Build and send the request.
        Response::build()
            .sized_body(string.len(), Cursor::new(string))
            .header(ContentType::new("application", "json"))
            .status(status)
            .ok()
    }
}
