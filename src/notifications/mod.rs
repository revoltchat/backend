pub mod events;
pub mod pubsub;
pub mod state;
pub mod ws;

pub fn send_message<U: Into<Option<Vec<String>>>, G: Into<Option<String>>>(
    users: U,
    guild: G,
    data: events::Notification,
) -> bool {
    let users = users.into();
    let guild = guild.into();

    if pubsub::send_message(users.clone(), guild.clone(), data) {
        state::send_message(users, guild, "bruh".to_string());

        true
    } else {
        false
    }
}
