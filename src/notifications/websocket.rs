use crate::database::*;
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

    let session: Arc<Mutex<Option<Session>>> = Arc::new(Mutex::new(None));
    let mutex_generator = || { session.clone() };
    let fwd = rx.map(Ok).forward(write);
    let incoming = read.try_for_each(async move |msg| {
        let mutex = mutex_generator();
        //dbg!(&mutex.lock().unwrap());

        if let Message::Text(text) = msg {
            if let Ok(notification) = serde_json::from_str::<ServerboundNotification>(&text) {
                match notification {
                    ServerboundNotification::Authenticate(new_session) => {
                        {
                            if mutex.lock().unwrap().is_some() {
                                send(ClientboundNotification::Error(
                                    WebSocketError::AlreadyAuthenticated,
                                ));
                                
                                return Ok(())
                            }
                        }

                        if let Ok(validated_session) = Auth::new(get_collection("accounts"))
                            .verify_session(new_session)
                            .await {
                            let id = validated_session.user_id.clone();
                            if let Ok(user) = (
                                Ref {
                                    id: id.clone()
                                }
                            )
                            .fetch_user()
                            .await {
                                let was_online = is_online(&id);
                                {
                                    match USERS.write() {
                                        Ok(mut map) => {
                                            map.insert(id.clone(), addr);
                                        }
                                        Err(_) => {
                                            send(ClientboundNotification::Error(
                                                WebSocketError::InternalError { at: "Writing users map.".to_string() },
                                            ));

                                            return Ok(())
                                        }
                                    }
                                }

                                *mutex.lock().unwrap() = Some(validated_session);

                                if let Err(_) = subscriptions::generate_subscriptions(&user).await {
                                    send(ClientboundNotification::Error(
                                        WebSocketError::InternalError { at: "Generating subscriptions.".to_string() },
                                    ));

                                    return Ok(())
                                }

                                send(ClientboundNotification::Authenticated);

                                match super::payload::generate_ready(user).await {
                                    Ok(payload) => {
                                        send(payload);

                                        if !was_online {
                                            ClientboundNotification::UserPresence {
                                                id: id.clone(),
                                                online: true
                                            }
                                            .publish(id)
                                            .await
                                            .ok();
                                        }
                                    }
                                    Err(_) => {
                                        send(ClientboundNotification::Error(
                                            WebSocketError::InternalError { at: "Generating payload.".to_string() },
                                        ));

                                        return Ok(())
                                    }
                                }
                            } else {
                                send(ClientboundNotification::Error(
                                    WebSocketError::OnboardingNotFinished,
                                ));
                            }
                        } else {
                            send(ClientboundNotification::Error(
                                WebSocketError::InvalidSession,
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    });

    pin_mut!(fwd, incoming);
    future::select(fwd, incoming).await;

    info!("User {} disconnected.", &addr);
    CONNECTIONS.lock().unwrap().remove(&addr);

    let session = session.lock().unwrap();
    if let Some(session) = session.as_ref() {
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

pub fn is_online(user: &String) -> bool {
    USERS.read().unwrap().get_left(&user).is_some()
}
