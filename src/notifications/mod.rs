use once_cell::sync::OnceCell;
use std::sync::mpsc::{channel, Sender};
use std::thread;

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

    if pubsub::send_message(users.clone(), guild.clone(), data.clone()) {
        state::send_message(users, guild, data.serialize());

        true
    } else {
        false
    }
}

struct NotificationArguments {
    users: Option<Vec<String>>,
    guild: Option<String>,
    data: events::Notification,
}

static mut SENDER: OnceCell<Sender<NotificationArguments>> = OnceCell::new();

pub fn start_worker() {
    let (sender, receiver) = channel();
    unsafe {
        SENDER.set(sender).unwrap();
    }

    thread::spawn(move || {
        while let Ok(data) = receiver.recv() {
            send_message(data.users, data.guild, data.data);
        }
    });
}

pub fn send_message_threaded<U: Into<Option<Vec<String>>>, G: Into<Option<String>>>(
    users: U,
    guild: G,
    data: events::Notification,
) -> bool {
    unsafe {
        SENDER
            .get()
            .unwrap()
            .send(NotificationArguments {
                users: users.into(),
                guild: guild.into(),
                data,
            })
            .is_ok()
    }
}
