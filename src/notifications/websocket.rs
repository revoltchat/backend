use crate::database::get_collection;
use crate::database::guards::reference::Ref;
use crate::util::variables::WS_HOST;

use super::subscriptions;

use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::tungstenite::Message;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::stream::TryStreamExt;
use futures::{pin_mut, prelude::*};
use hive_pubsub::PubSub;
use log::{debug, info};
use many_to_many::ManyToMany;
use rauth::auth::{Auth, Session};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};

use super::{
    events::{ClientboundNotification, ServerboundNotification, WebSocketError},
    hive::get_hive,
};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

lazy_static! {
    static ref CONNECTIONS: PeerMap = Arc::new(Mutex::new(HashMap::new()));
    static ref USERS: Arc<RwLock<ManyToMany<String, SocketAddr>>> =
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

    let (write, read) = ws_stream.split();
    let (tx, rx) = unbounded();
    CONNECTIONS.lock().unwrap().insert(addr, tx.clone());

    let send = |notification: ClientboundNotification| {
        if let Ok(response) = serde_json::to_string(&notification) {
            if let Err(_) = tx.unbounded_send(Message::Text(response)) {
                debug!("Failed unbounded_send to websocket stream.");
            }
        }
    };

    let mut session: Option<Session> = None;
    let fwd = rx.map(Ok).forward(write);
    let incoming = read.try_for_each(|msg| {
        if let Message::Text(text) = msg {
            if let Ok(notification) = serde_json::from_str::<ServerboundNotification>(&text) {
                match notification {
                    ServerboundNotification::Authenticate(new_session) => {
                        if session.is_some() {
                            send(ClientboundNotification::Error(
                                WebSocketError::AlreadyAuthenticated,
                            ));
                            return future::ok(());
                        }

                        match task::block_on(
                            Auth::new(get_collection("accounts")).verify_session(new_session),
                        ) {
                            Ok(validated_session) => {
                                match task::block_on(
                                    Ref {
                                        id: validated_session.user_id.clone(),
                                    }
                                    .fetch_user(),
                                ) {
                                    Ok(user) => {
                                        if let Ok(mut map) = USERS.write() {
                                            map.insert(validated_session.user_id.clone(), addr);
                                            session = Some(validated_session);
                                            if let Ok(_) = task::block_on(
                                                subscriptions::generate_subscriptions(&user),
                                            ) {
                                                send(ClientboundNotification::Authenticated);
                                                send(ClientboundNotification::Ready { user });
                                            } else {
                                                send(ClientboundNotification::Error(
                                                    WebSocketError::InternalError,
                                                ));
                                            }
                                        } else {
                                            send(ClientboundNotification::Error(
                                                WebSocketError::InternalError,
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        send(ClientboundNotification::Error(
                                            WebSocketError::OnboardingNotFinished,
                                        ));
                                    }
                                }
                            }
                            Err(_) => {
                                send(ClientboundNotification::Error(
                                    WebSocketError::InvalidSession,
                                ));
                            }
                        }
                    }
                }
            }
        }

        future::ok(())
    });

    pin_mut!(fwd, incoming);
    future::select(fwd, incoming).await;

    info!("User {} disconnected.", &addr);
    CONNECTIONS.lock().unwrap().remove(&addr);

    if let Some(session) = session {
        let mut users = USERS.write().unwrap();
        users.remove(&session.user_id, &addr);
        if users.get_left(&session.user_id).is_none() {
            get_hive().drop_client(&session.user_id).unwrap();
        }
    }
}

pub fn publish(ids: Vec<String>, notification: ClientboundNotification) {
    let mut targets = vec![];
    {
        let users = USERS.read().unwrap();
        for id in ids {
            // Block certain notifications from reaching users that aren't meant to see them.
            if let ClientboundNotification::UserRelationship { id: user_id, .. } = &notification {
                if &id != user_id {
                    continue;
                }
            }

            if let Some(mut arr) = users.get_left(&id) {
                targets.append(&mut arr);
            }
        }
    }

    let msg = Message::Text(serde_json::to_string(&notification).unwrap());

    let connections = CONNECTIONS.lock().unwrap();
    for target in targets {
        if let Some(conn) = connections.get(&target) {
            if let Err(_) = conn.unbounded_send(msg.clone()) {
                debug!("Failed unbounded_send.");
            }
        }
    }
}
