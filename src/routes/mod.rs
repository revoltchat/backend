pub use rocket::http::Status;
pub use rocket::response::Redirect;
use rocket::Rocket;

mod root;
mod users;
mod guild;
mod onboard;
mod channels;

pub fn mount(rocket: Rocket) -> Rocket {
    rocket
        .mount("/", routes![root::root, root::teapot])
        .mount("/onboard", onboard::routes())
        .mount("/users", users::routes())
        .mount("/channels", channels::routes())
        .mount("/guild", guild::routes())

        /*.mount(
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
        )*/
}
