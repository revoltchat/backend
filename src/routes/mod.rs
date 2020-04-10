pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::Rocket;
use rocket_contrib::json::JsonValue;

use crate::database::Permission;

pub mod account;
pub mod channel;
pub mod guild;
pub mod root;
pub mod user;

#[derive(Responder)]
pub enum Response {
    #[response()]
    Result(Status),
    #[response()]
    Success(JsonValue),
    #[response()]
    Redirect(Redirect),
    #[response(status = 400)]
    BadRequest(JsonValue),
    #[response(status = 401)]
    Unauthorized(JsonValue),
    #[response(status = 401)]
    LackingPermission(Permission),
    #[response(status = 404)]
    NotFound(JsonValue),
    #[response(status = 406)]
    NotAcceptable(JsonValue),
    #[response(status = 409)]
    Conflict(JsonValue),
    #[response(status = 410)]
    Gone(JsonValue),
    #[response(status = 422)]
    UnprocessableEntity(JsonValue),
    #[response(status = 429)]
    TooManyRequests(JsonValue),
    #[response(status = 500)]
    InternalServerError(JsonValue),
}

use rocket::http::ContentType;
use rocket::request::Request;
use std::io::Cursor;

impl<'a> rocket::response::Responder<'a> for Permission {
    fn respond_to(self, _: &Request) -> rocket::response::Result<'a> {
        rocket::response::Response::build()
            .header(ContentType::JSON)
            .sized_body(Cursor::new(format!(
                "{{\"error\":\"Lacking {:?} permission.\"}}",
                self
            )))
            .ok()
    }
}

pub fn mount(rocket: Rocket) -> Rocket {
    rocket
        .mount("/api", routes![root::root])
        .mount(
            "/api/account",
            routes![
                account::create,
                account::verify_email,
                account::resend_email,
                account::login,
                account::token
            ],
        )
        .mount(
            "/api/users",
            routes![
                user::me,
                user::user,
                user::lookup,
                user::dms,
                user::dm,
                user::get_friends,
                user::get_friend,
                user::add_friend,
                user::remove_friend,
                user::block_user,
                user::unblock_user
            ],
        )
        .mount(
            "/api/channels",
            routes![
                channel::create_group,
                channel::channel,
                channel::add_member,
                channel::remove_member,
                channel::delete,
                channel::messages,
                channel::get_message,
                channel::send_message,
                channel::edit_message,
                channel::delete_message
            ],
        )
        .mount(
            "/api/guild",
            routes![
                guild::my_guilds,
                guild::guild,
                guild::create_guild,
                guild::fetch_members,
                guild::fetch_member,
                guild::kick_member
            ],
        )
}
