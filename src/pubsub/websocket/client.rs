use super::state;
use crate::database::get_collection;
use crate::pubsub::hive;

use log::{error, info};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde_json::{from_str, json, Value};
use ulid::Ulid;
use ws::{CloseCode, Error, Handler, Handshake, Message, Result, Sender};

pub struct Client {
    id: String,
    sender: Sender,
    user_id: Option<String>,
}

impl Client {
    pub fn new(sender: Sender) -> Client {
        Client {
            id: Ulid::new().to_string(),
            user_id: None,
            sender,
        }
    }
}

impl Handler for Client {
    fn on_open(&mut self, handshake: Handshake) -> Result<()> {
        info!("Client connected. [{}] {:?}", self.id, handshake.peer_addr);

        Ok(())
    }

    // Client sends { "type": "authenticate", "token": token }.
    // Receives { "type": "authorised" } and waits.
    // Client then receives { "type": "ready", "data": payload }.
    // If at any point we hit an error, send { "type": "error", "error": error }.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Text(text) = msg {
            if let Ok(data) = from_str(&text) as std::result::Result<Value, _> {
                if let Value::String(packet_type) = &data["type"] {
                    if packet_type == "authenticate" {
                        if self.user_id.is_some() {
                            return self.sender.send(
                                json!({
                                    "type": "error",
                                    "error": "Already authenticated!"
                                })
                                .to_string(),
                            );
                        } else if let Value::String(token) = &data["token"] {
                            let user = get_collection("users").find_one(
                                doc! {
                                    "access_token": token
                                },
                                FindOneOptions::builder()
                                    .projection(doc! { "_id": 1 })
                                    .build(),
                            );

                            if let Ok(result) = user {
                                if let Some(doc) = result {
                                    self.sender.send(
                                        json!({
                                            "type": "authorised"
                                        })
                                        .to_string(),
                                    )?;

                                    // FIXME: fetch above when we switch to new token system
                                    // or auth cache system, something like that
                                    let user = crate::database::user::fetch_user(
                                        doc.get_str("_id").unwrap(),
                                    )
                                    .unwrap()
                                    .unwrap(); // this should be guranteed, I think, maybe? I'm getting rid of it later. FIXME

                                    self.user_id = Some(user.id.clone());

                                    match user.create_payload() {
                                        Ok(payload) => {
                                            // ! Grab the ids from the payload,
                                            // ! there's probably a better way to
                                            // ! do this. I'll rewrite it at some point.
                                            let mut ids = vec![
                                                self.user_id.as_ref().unwrap().clone()
                                            ];

                                            {
                                                // This is bad code. But to be fair
                                                // it should work just fine.
                                                for user in payload.get("users").unwrap().as_array().unwrap() {
                                                    ids.push(user.as_object().unwrap().get("id").unwrap().as_str().unwrap().to_string());
                                                }

                                                for channel in payload.get("channels").unwrap().as_array().unwrap() {
                                                    ids.push(channel.as_object().unwrap().get("id").unwrap().as_str().unwrap().to_string());
                                                }

                                                for guild in payload.get("guilds").unwrap().as_array().unwrap() {
                                                    ids.push(guild.as_object().unwrap().get("id").unwrap().as_str().unwrap().to_string());
                                                }
                                            }

                                            if let Err(err) = hive::subscribe(self.user_id.as_ref().unwrap().clone(), ids) {
                                                self.sender.send(
                                                    json!({
                                                        "type": "warn",
                                                        "error": "Failed to subscribe you to the Hive. You may not receive all notifications."
                                                    })
                                                    .to_string(),
                                                )?;
                                            }

                                            self.sender.send(
                                                json!({
                                                    "type": "ready",
                                                    "data": payload
                                                })
                                                .to_string(),
                                            )?;

                                            if state::accept(
                                                self.id.clone(),
                                                self.user_id.as_ref().unwrap().clone(),
                                                self.sender.clone(),
                                            )
                                            .is_err()
                                            {
                                                self.sender.send(
                                                    json!({
                                                        "type": "warn",
                                                        "error": "Failed to accept your connection. You will not receive any notifications."
                                                    })
                                                    .to_string(),
                                                )?;
                                            }
                                        }
                                        Err(error) => {
                                            error!("Failed to create payload! {}", error);
                                            self.sender.send(
                                                json!({
                                                    "type": "error",
                                                    "error": "Failed to create payload."
                                                })
                                                .to_string(),
                                            )?;
                                        }
                                    }
                                } else {
                                    self.sender.send(
                                        json!({
                                            "type": "error",
                                            "error": "Invalid token."
                                        })
                                        .to_string(),
                                    )?;
                                }
                            } else {
                                self.sender.send(
                                    json!({
                                        "type": "error",
                                        "error": "Failed to fetch from database."
                                    })
                                    .to_string(),
                                )?;
                            }
                        } else {
                            self.sender.send(
                                json!({
                                    "type": "error",
                                    "error": "Missing token."
                                })
                                .to_string(),
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, reason: &str) {
        info!("Client disconnected. [{}] {}", self.id, reason);
        if let Err(error) = state::drop(&self.id) {
            error!("Also failed to drop client from state! {}", error);
        }
    }

    fn on_error(&mut self, err: Error) {
        error!(
            "A client disconnected due to an error. [{}] {}",
            self.id, err
        );
    }
}
