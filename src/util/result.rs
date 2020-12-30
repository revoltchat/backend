use json;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde::Serialize;
use snafu::Snafu;
use std::io::Cursor;
use validator::ValidationErrors;

#[derive(Serialize, Debug, Snafu)]
#[serde(tag = "type")]
pub enum Error {
    #[snafu(display("This error has not been labelled."))]
    LabelMe,

    // ? Onboarding related errors.
    #[snafu(display("Already finished onboarding."))]
    AlreadyOnboarded,

    // ? User related errors.
    #[snafu(display("Username has already been taken."))]
    UsernameTaken,
    #[snafu(display("This user does not exist!"))]
    UnknownUser,
    #[snafu(display("Already friends with this user."))]
    AlreadyFriends,
    #[snafu(display("Already sent a request to this user."))]
    AlreadySentRequest,
    #[snafu(display("You have blocked this user."))]
    Blocked,
    #[snafu(display("You have been blocked by this user."))]
    BlockedByOther,

    // ? General errors.
    #[snafu(display("Failed to validate fields."))]
    FailedValidation { error: ValidationErrors },
    #[snafu(display("Encountered a database error."))]
    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },
    #[snafu(display("Internal server error."))]
    InternalError,
    #[snafu(display("This request had no effect."))]
    NoEffect,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// HTTP response builder for Error enum
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self {
            Error::LabelMe => Status::InternalServerError,

            Error::AlreadyOnboarded => Status::Forbidden,

            Error::UnknownUser => Status::NotFound,
            Error::UsernameTaken => Status::Conflict,
            Error::AlreadyFriends => Status::Conflict,
            Error::AlreadySentRequest => Status::Conflict,
            Error::Blocked => Status::Conflict,
            Error::BlockedByOther => Status::Forbidden,

            Error::FailedValidation { .. } => Status::UnprocessableEntity,
            Error::DatabaseError { .. } => Status::InternalServerError,
            Error::InternalError => Status::InternalServerError,
            Error::NoEffect => Status::Ok,
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
