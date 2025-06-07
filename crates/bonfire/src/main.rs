use std::{env, sync::Arc};

use async_std::net::TcpListener;
use revolt_presence::clear_region;
use once_cell::sync::OnceCell;
use revolt_database::voice::VoiceClient;

#[macro_use]
extern crate log;

pub mod config;
pub mod events;

mod database;
mod websocket;

pub static VOICE_CLIENT: OnceCell<Arc<VoiceClient>> = OnceCell::new();

pub fn get_voice_client() -> Arc<VoiceClient> {
    VOICE_CLIENT
        .get()
        .expect("get_voice_client called before set")
        .clone()
}

#[async_std::main]
async fn main() {
    // Configure requirements for Bonfire.
    revolt_config::configure!(events);
    database::connect().await;

    // Clean up the current region information.
    let no_clear_region = env::var("NO_CLEAR_PRESENCE").unwrap_or_else(|_| "0".into()) == "1";
    if !no_clear_region {
        clear_region(None).await;
    }

    VOICE_CLIENT.set(Arc::new(VoiceClient::from_revolt_config().await)).unwrap();

    // Setup a TCP listener to accept WebSocket connections on.
    // By default, we bind to port 14703 on all interfaces.
    let bind = env::var("HOST").unwrap_or_else(|_| "0.0.0.0:14703".into());
    info!("Listening on host {bind}");
    let try_socket = TcpListener::bind(bind).await;
    let listener = try_socket.expect("Failed to bind");

    // Start accepting new connections and spawn a client for each connection.
    while let Ok((stream, addr)) = listener.accept().await {
        async_std::task::spawn(async move {
            info!("User connected from {addr:?}");
            websocket::client(database::get_db(), stream, addr).await;
            info!("User disconnected from {addr:?}");
        });
    }
}
