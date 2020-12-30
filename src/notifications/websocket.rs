use crate::{database::entities::User, util::variables::WS_HOST};

use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::tungstenite::Message;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::{pin_mut, prelude::*};
use log::info;
use many_to_many::ManyToMany;
use rauth::auth::Session;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::from_utf8;
use std::sync::{Arc, Mutex, RwLock};
use ulid::Ulid;

use super::events::ServerboundNotification;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

lazy_static! {
    static ref CONNECTIONS: PeerMap = Arc::new(Mutex::new(HashMap::new()));
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
    let addr = stream
        .peer_addr()
        .expect("Connected streams should have a peer address.");
    let ws_stream = async_tungstenite::accept_async(stream)
        .await
        .expect("Error during websocket handshake.");

    info!("User established WebSocket connection from {}.", &addr);

    let id = Ulid::new().to_string();
    let (write, read) = ws_stream.split();

    let (tx, rx) = unbounded();
    CONNECTIONS.lock().unwrap().insert(addr, tx);

    let session: Option<Session> = None;
    let user: Option<User> = None;

    let fwd = rx.map(Ok).forward(write);
    let reading = read.for_each(|message| async {
        let data = message.unwrap().into_data();
        // if you mess with the data, you get the bazooki
        let string = from_utf8(&data).unwrap();

        if let Ok(notification) = serde_json::from_str::<ServerboundNotification>(string) {
            match notification {
                ServerboundNotification::Authenticate(a) => {
                    dbg!(a);
                }
            }
        }
    });

    pin_mut!(fwd, reading);
    future::select(fwd, reading).await;

    println!("User {} disconnected.", &addr);
}
