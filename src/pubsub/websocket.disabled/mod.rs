use crate::util::variables::WS_HOST;

use log::{error, info};
use std::thread;
use ws::listen;

mod client;
mod state;

pub use state::publish;

pub fn launch_server() {
    thread::spawn(|| {
        if listen(WS_HOST.to_string(), |sender| client::Client::new(sender)).is_err() {
            error!(
                "Failed to listen for WebSocket connections on {:?}!",
                *WS_HOST
            );
        } else {
            info!("Listening for WebSocket connections on {:?}", *WS_HOST);
        }
    });
}
