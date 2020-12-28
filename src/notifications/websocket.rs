use crate::util::variables::WS_HOST;

use log::info;
use async_std::task;
use futures::prelude::*;
use async_std::net::{TcpListener, TcpStream};

pub async fn launch_server() {
    let try_socket = TcpListener::bind(WS_HOST.to_string()).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", *WS_HOST);

    while let Ok((stream, _)) = listener.accept().await {
        task::spawn(accept(stream));
    }
}

async fn accept(stream: TcpStream) {
    let addr = stream.peer_addr().expect("Connected streams should have a peer address.");
    let ws_stream = async_tungstenite::accept_async(stream)
        .await
        .expect("Error during websocket handshake.");

    info!("User established WebSocket connection from {}.", addr);

    let (write, read) = ws_stream.split();
    read.forward(write).await.expect("Failed to forward message")
}
