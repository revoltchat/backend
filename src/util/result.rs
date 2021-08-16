use json;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde::Serialize;
use std::io::Cursor;
use validator::ValidationErrors;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Error {
    LabelMe,

    // ? Onboarding related errors.
    AlreadyOnboarded,

    // ? User related errors.
    UsernameTaken,
    UnknownUser,
    AlreadyFriends,
    AlreadySentRequest,
    Blocked,
    BlockedByOther,
    NotFriends,

    // ? Channel related errors.
    UnknownChannel,
    UnknownAttachment,
    UnknownMessage,
    CannotEditMessage,
    CannotJoinCall,
    TooManyAttachments,
    TooManyReplies,
    EmptyMessage,
    CannotRemoveYourself,
    GroupTooLarge {
        max: usize,
    },
    AlreadyInGroup,
    NotInGroup,

    // ? Server related errors.
    UnknownServer,
    InvalidRole,
    Banned,

    // ? Bot related errors.
    ReachedMaximumBots,
    IsBot,
    BotIsPrivate,

    // ? General errors.
    TooManyIds,
    FailedValidation {
        error: ValidationErrors,
    },
    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },
    InternalError,
    MissingPermission,
    InvalidOperation,
    InvalidCredentials,
    DuplicateNonce,
    VosoUnavailable,
    NotFound,
    NoEffect,
}

pub struct EmptyResponse;
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl<'r> Responder<'r> for EmptyResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build()
            .status(rocket::http::Status { code: 204 })
            .ok()
    }
}

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
            Error::UnknownMessage => Status::NotFound,
            Error::UnknownAttachment => Status::BadRequest,
            Error::CannotEditMessage => Status::Forbidden,
            Error::CannotJoinCall => Status::BadRequest,
            Error::TooManyAttachments => Status::BadRequest,
            Error::TooManyReplies => Status::BadRequest,
            Error::EmptyMessage => Status::UnprocessableEntity,
            Error::CannotRemoveYourself => Status::BadRequest,
            Error::GroupTooLarge { .. } => Status::Forbidden,
            Error::AlreadyInGroup => Status::Conflict,
            Error::NotInGroup => Status::NotFound,

            Error::UnknownServer => Status::NotFound,
            Error::InvalidRole => Status::NotFound,
            Error::Banned => Status::Forbidden,

            Error::ReachedMaximumBots => Status::BadRequest,
            Error::IsBot => Status::BadRequest,
            Error::BotIsPrivate => Status::Forbidden,

            Error::FailedValidation { .. } => Status::UnprocessableEntity,
            Error::DatabaseError { .. } => Status::InternalServerError,
            Error::InternalError => Status::InternalServerError,
            Error::MissingPermission => Status::Forbidden,
            Error::InvalidOperation => Status::BadRequest,
            Error::TooManyIds => Status::BadRequest,
            Error::InvalidCredentials => Status::Forbidden,
            Error::DuplicateNonce => Status::Conflict,
            Error::VosoUnavailable => Status::BadRequest,
            Error::NotFound => Status::NotFound,
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
