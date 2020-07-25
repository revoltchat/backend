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

impl<'a> rocket::response::Responder<'a> for Permission {
    fn respond_to(self, _: &Request) -> rocket::response::Result<'a> {
        rocket::response::Response::build()
            .header(ContentType::JSON)
            .sized_body(Cursor::new(format!(
                "{{\"error\":\"Lacking permission: {:?}.\",\"permission\":{}}}",
                self, self as u32,
            )))
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
                account::create_preflight,
                account::verify_email_preflight,
                account::resend_email_preflight,
                account::login_preflight,
                account::token_preflight,
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
                user::user_preflight,
                user::lookup_preflight,
                user::dms_preflight,
                user::dm_preflight,
                user::friend_preflight,
                user::block_user_preflight,
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
                channel::create_group_preflight,
                channel::channel_preflight,
                channel::member_preflight,
                channel::messages_preflight,
                channel::message_preflight,
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
                guild::guild_preflight,
                guild::create_channel_preflight,
                guild::create_invite_preflight,
                guild::remove_invite_preflight,
                guild::fetch_invites_preflight,
                guild::invite_preflight,
                guild::create_guild_preflight,
                guild::fetch_members_preflight,
                guild::fetch_member_preflight,
                guild::ban_member_preflight,
            ],
        )
}
