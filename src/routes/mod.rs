use rocket::Rocket;

pub mod account;
pub mod channel;
pub mod guild;
pub mod root;
pub mod user;

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
        .mount("/api/guild", routes![guild::create_guild])
}
