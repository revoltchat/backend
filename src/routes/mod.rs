pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::Rocket;
use rocket_contrib::json::JsonValue;

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
                user::remove_friend
            ],
        )
        .mount(
            "/api/channels",
            routes![
                channel::channel,
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
            routes![guild::my_guilds, guild::guild, guild::create_guild],
        )
}
