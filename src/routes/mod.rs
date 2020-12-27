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
    #[response(status = 207)]
    PartialStatus(JsonValue),
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
    #[response(status = 418)]
    Teapot(JsonValue),
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

use rocket::response::{Responder, Result};

impl<'a> Responder<'a, 'static> for Permission {
    fn respond_to(self, _: &Request) -> Result<'static> {
        let body = format!(
            "{{\"error\":\"Lacking permission: {:?}.\",\"permission\":{}}}",
            self, self as u32,
        );

        rocket::response::Response::build()
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

pub fn mount(rocket: Rocket) -> Rocket {
    rocket
        .mount("/", routes![root::root, root::teapot])
        .mount(
            "/account",
            routes![
                account::create,
                account::verify_email,
                account::resend_email,
                account::login,
                account::token,
            ],
        )
        .mount(
            "/users",
            routes![
                user::me,
                user::user,
                user::query,
                user::dms,
                user::dm,
                user::get_friends,
                user::get_friend,
                user::add_friend,
                user::remove_friend,
                user::block_user,
                user::unblock_user,
            ],
        )
        .mount(
            "/channels",
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
                channel::delete_message,
            ],
        )
        .mount(
            "/guild",
            routes![
                guild::my_guilds,
                guild::guild,
                guild::remove_guild,
                guild::create_channel,
                guild::create_invite,
                guild::remove_invite,
                guild::fetch_invites,
                guild::fetch_invite,
                guild::use_invite,
                guild::create_guild,
                guild::fetch_members,
                guild::fetch_member,
                guild::kick_member,
                guild::ban_member,
                guild::unban_member,
            ],
        )
}
