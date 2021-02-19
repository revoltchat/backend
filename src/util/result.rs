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
    #[snafu(display("Not friends with target user."))]
    NotFriends,

    // ? Channel related errors.
    #[snafu(display("This channel does not exist!"))]
    UnknownChannel,
    #[snafu(display("Attachment does not exist!"))]
    UnknownAttachment,
    #[snafu(display("Cannot edit someone else's message."))]
    CannotEditMessage,
    #[snafu(display("Cannot remove yourself from a group, use delete channel instead."))]
    CannotRemoveYourself,
    #[snafu(display("Group size is too large."))]
    GroupTooLarge { max: usize },
    #[snafu(display("User already part of group."))]
    AlreadyInGroup,
    #[snafu(display("User is not part of the group."))]
    NotInGroup,

    // ? General errors.
    #[snafu(display("Trying to fetch too much data."))]
    TooManyIds,
    #[snafu(display("Failed to validate fields."))]
    FailedValidation { error: ValidationErrors },
    #[snafu(display("Encountered a database error."))]
    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },
    #[snafu(display("Internal server error."))]
    InternalError,
    #[snafu(display("Missing permission."))]
    MissingPermission,
    #[snafu(display("Operation cannot be performed on this object."))]
    InvalidOperation,
    #[snafu(display("Already created an object with this nonce."))]
    DuplicateNonce,
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
            Error::NotFriends => Status::Forbidden,

            Error::UnknownChannel => Status::NotFound,
            Error::UnknownAttachment => Status::BadRequest,
            Error::CannotEditMessage => Status::Forbidden,
            Error::CannotRemoveYourself => Status::BadRequest,
            Error::GroupTooLarge { .. } => Status::Forbidden,
            Error::AlreadyInGroup => Status::Conflict,
            Error::NotInGroup => Status::NotFound,

            Error::FailedValidation { .. } => Status::UnprocessableEntity,
            Error::DatabaseError { .. } => Status::InternalServerError,
            Error::InternalError => Status::InternalServerError,
            Error::MissingPermission => Status::Forbidden,
            Error::InvalidOperation => Status::BadRequest,
            Error::TooManyIds => Status::BadRequest,
            Error::DuplicateNonce => Status::Conflict,
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
