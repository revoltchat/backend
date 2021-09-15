use crate::database::*;
use crate::notifications::events::{AuthType, BotAuth};
use crate::util::variables::WS_HOST;

use super::subscriptions;

use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::tungstenite::{Message, handshake::server};
use futures::channel::{oneshot, mpsc::{unbounded, UnboundedSender}};
use futures::stream::TryStreamExt;
use futures::{pin_mut, prelude::*};
use hive_pubsub::PubSub;
use log::{debug, info};
use many_to_many::ManyToMany;
use mongodb::bson::doc;
use rauth::{
    auth::{Auth},
    options::Options,
};
use rmp_serde;
use url::Url;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};

use super::{
    events::{ClientboundNotification, ServerboundNotification, WebSocketError},
    hive::get_hive,
};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, (Tx, MSGFormat)>>>;

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

#[derive(Debug, Clone)]
enum MSGFormat {
    JSON,
    MSGPACK
}

struct HeaderCallback {
    sender: oneshot::Sender<MSGFormat>
}

impl server::Callback for HeaderCallback {
    fn on_request(self, request: &server::Request, response: server::Response) -> Result<server::Response, server::ErrorResponse> {
        // we dont get some of the data sometimes so im generating a fake url with the only data we actually need
        let url = format!("ws://example.com?{}", request.uri().query().unwrap_or("?format=json"));
        let mut query: HashMap<_, _> = url.parse::<Url>().unwrap().query_pairs().into_owned().collect();  // should be safe to use unwrap here as we just made the url ourself
        let format_query: Option<String> = query.remove("format");

        let format = match format_query.as_deref().unwrap_or("json") {
            "msgpack" => MSGFormat::MSGPACK,
            "json" => MSGFormat::JSON,
            _ => panic!("unknown format")  // TODO: not use panic
        };

        self.sender.send(format).unwrap();  // TODO: not use unwrap
        Ok(response)
    }
}

async fn accept(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("Connected streams should have a peer address.");
    let (sender, receiver) = oneshot::channel::<MSGFormat>();
    
    let ws_stream = async_tungstenite::accept_hdr_async_with_config(stream, HeaderCallback { sender }, None)
        .await
        .expect("Error during websocket handshake.");

    let msg_format = receiver.await.unwrap();  // TODO: not use unwrap

    info!("User established WebSocket connection from {}.", &addr);

    let (write, read) = ws_stream.split();
    let (tx, rx) = unbounded();
    CONNECTIONS.lock().unwrap().insert(addr, (tx.clone(), msg_format.clone()));

    let send = |notification: ClientboundNotification| {
        let res = match msg_format {
            MSGFormat::JSON => match serde_json::to_string(&notification) {
                Ok(s) => Message::Text(s),
                Err(_) => return
            }
            MSGFormat::MSGPACK => match rmp_serde::to_vec_named(&notification) {
                Ok(v) => Message::Binary(v),
                Err(_) => return
            }
        };

        if let Err(_) = tx.unbounded_send(res) {
            debug!("Failed unbounded_send to websocket stream.");
        }
    };

    let user_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let mutex_generator = || user_id.clone();
    let fwd = rx.map(Ok).forward(write);
    let incoming = read.try_for_each(async move |msg| {
        let mutex = mutex_generator();

        let maybe_decoded = match msg {
            Message::Text(text) => serde_json::from_str::<ServerboundNotification>(&text).map_err(|e| e.to_string()),
            Message::Binary(vec) => rmp_serde::decode::from_read::<&[u8], ServerboundNotification>(vec.as_slice()).map_err(|e| e.to_string()),
            Message::Ping(vec) => Ok(ServerboundNotification::Ping { data: vec }),
            _ => return Ok(())
        };

        let notification = match maybe_decoded {
            Err(why) => {
                send(ClientboundNotification::Error(
                    WebSocketError::MalformedData {
                        msg: why.to_string()
                }));
                return Ok(())
            },
            Ok(n) => n
        };

        match notification {
            ServerboundNotification::Authenticate(auth) => {
                {
                    if mutex.lock().unwrap().is_some() {
                        send(ClientboundNotification::Error(
                            WebSocketError::AlreadyAuthenticated,
                        ));

                        return Ok(());
                    }
                }

                if let Some(id) = match auth {
                    AuthType::User(new_session) => {
                        if let Ok(validated_session) =
                        Auth::new(get_collection("accounts"), Options::new())
                            .verify_session(new_session)
                            .await
                        {
                            Some(validated_session.user_id.clone())
                        } else {
                            None
                        }
                    }
                    AuthType::Bot(BotAuth { token }) => {
                        if let Ok(doc) = get_collection("bots")
                            .find_one(
                                doc! { "token": token },
                                None
                            ).await {
                                if let Some(doc) = doc {
                                    Some(doc.get_str("_id").unwrap().to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                    }
                } {
                    if let Ok(user) = (Ref { id: id.clone() }).fetch_user().await {
                        let is_invisible = if let Some(status) = &user.status {
                            if let Some(presence) = &status.presence {
                                presence == &Presence::Invisible
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        let was_online = is_online(&id);

                        {
                            match USERS.write() {
                                Ok(mut map) => {
                                    map.insert(id.clone(), addr);
                                }
                                Err(_) => {
                                    send(ClientboundNotification::Error(
                                        WebSocketError::InternalError {
                                            at: "Writing users map.".to_string(),
                                        },
                                    ));

                                    return Ok(());
                                }
                            }
                        }

                        *mutex.lock().unwrap() = Some(id.clone());

                        if let Err(_) = subscriptions::generate_subscriptions(&user).await {
                            send(ClientboundNotification::Error(
                                WebSocketError::InternalError {
                                    at: "Generating subscriptions.".to_string(),
                                },
                            ));

                            return Ok(());
                        }

                        send(ClientboundNotification::Authenticated);

                        match super::payload::generate_ready(user).await {
                            Ok(payload) => {
                                send(payload);

                                if !was_online && !is_invisible {
                                    ClientboundNotification::UserUpdate {
                                        id: id.clone(),
                                        data: json!({
                                            "online": true
                                        }),
                                        clear: None
                                    }
                                    .publish_as_user(id);
                                }
                            }
                            Err(_) => {
                                send(ClientboundNotification::Error(
                                    WebSocketError::InternalError {
                                        at: "Generating payload.".to_string(),
                                    },
                                ));

                                return Ok(());
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
            // ! TEMP: verify user part of channel
            // ! Could just run permission check here.
            ServerboundNotification::BeginTyping { channel } => {
                if mutex.lock().unwrap().is_some() {
                    let user = {
                        let mutex = mutex.lock().unwrap();
                        mutex.as_ref().unwrap().clone()
                    };

                    ClientboundNotification::ChannelStartTyping {
                        id: channel.clone(),
                        user,
                    }
                    .publish(channel);
                } else {
                    send(ClientboundNotification::Error(
                        WebSocketError::AlreadyAuthenticated,
                    ));

                    return Ok(());
                }
            }
            ServerboundNotification::EndTyping { channel } => {
                if mutex.lock().unwrap().is_some() {
                    let user = {
                        let mutex = mutex.lock().unwrap();
                        mutex.as_ref().unwrap().clone()
                    };

                    ClientboundNotification::ChannelStopTyping {
                        id: channel.clone(),
                        user,
                    }
                    .publish(channel);
                } else {
                    send(ClientboundNotification::Error(
                        WebSocketError::AlreadyAuthenticated,
                    ));

                    return Ok(());
                }
            }
            ServerboundNotification::Ping { data } => {
                info!("Ping received from User {}. Payload: {:?}", &addr, data);
            }
        }
        Ok(())
    });

    pin_mut!(fwd, incoming);
    future::select(fwd, incoming).await;

    info!("User {} disconnected.", &addr);
    CONNECTIONS.lock().unwrap().remove(&addr);

    let mut offline = None;
    {
        let user_id = user_id.lock().unwrap();
        if let Some(user_id) = user_id.as_ref() {
            let mut users = USERS.write().unwrap();
            users.remove(&user_id, &addr);
            if users.get_left(&user_id).is_none() {
                get_hive().drop_client(&user_id).unwrap();
                offline = Some(user_id.clone());
            }
        }
    }

    if let Some(id) = offline {
        ClientboundNotification::UserUpdate {
            id: id.clone(),
            data: json!({
                "online": false
            }),
            clear: None
        }
        .publish_as_user(id);
    }
}

pub fn publish(ids: Vec<String>, notification: ClientboundNotification) {
    let mut targets = vec![];
    {
        let users = USERS.read().unwrap();
        for id in ids {
            // Block certain notifications from reaching users that aren't meant to see them.
            match &notification {
                ClientboundNotification::UserRelationship { id: user_id, .. }
                | ClientboundNotification::UserSettingsUpdate { id: user_id, .. }
                | ClientboundNotification::ChannelAck { user: user_id, .. } => {
                    if &id != user_id {
                        continue;
                    }
                }
                _ => {}
            }

            if let Some(mut arr) = users.get_left(&id) {
                targets.append(&mut arr);
            }
        }
    }

    let json_msg = Message::Text(serde_json::to_string(&notification).unwrap());
    let msgpack_msg = Message::Binary(rmp_serde::to_vec_named(&notification).unwrap());

    let connections = CONNECTIONS.lock().unwrap();
    for target in targets {
        if let Some((conn, msg_format)) = connections.get(&target) {
            let msg = match msg_format {
                MSGFormat::JSON => json_msg.clone(),
                MSGFormat::MSGPACK => msgpack_msg.clone()
            };

            if let Err(_) = conn.unbounded_send(msg) {
                debug!("Failed unbounded_send.");
            }
        }
    }
}

pub fn is_online(user: &String) -> bool {
    USERS.read().unwrap().get_left(&user).is_some()
}
