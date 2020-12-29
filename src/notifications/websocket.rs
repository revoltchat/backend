use crate::util::variables::WS_HOST;

use log::info;
use ulid::Ulid;
use async_std::task;
use futures::prelude::*;
use std::str::from_utf8;
use std::sync::{Arc, RwLock};
use many_to_many::ManyToMany;
use std::collections::HashMap;
use futures::stream::SplitSink;
use async_tungstenite::WebSocketStream;
use async_tungstenite::tungstenite::Message;
use async_std::net::{TcpListener, TcpStream};

lazy_static! {
    static ref CONNECTIONS: Arc<RwLock<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref USERS: Arc<RwLock<ManyToMany<String, String>>> =
        Arc::new(RwLock::new(ManyToMany::new()));
}

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

    let id = Ulid::new().to_string();
    let (write, read) = ws_stream.split();

    CONNECTIONS
        .write()
        .unwrap()
        .insert(id, write);

    read
        .for_each(|message| async {
            let data = message.unwrap().into_data();
            // if you mess with the data, you get the bazooki
            let string = from_utf8(&data).unwrap();
            println!("{}", string);
        })
        .await;
}
