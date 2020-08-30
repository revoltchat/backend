use super::state::{self, StateResult};
use crate::util::variables::WS_HOST;

use serde_json::{from_str, json, Value};
use ulid::Ulid;
use ws::{listen, CloseCode, Error, Handler, Handshake, Message, Result, Sender};

struct Server {
    sender: Sender,
    user_id: Option<String>,
    id: String,
}

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        state::add_connection(self.id.clone(), self.sender.clone());
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Text(text) = msg {
            if let Ok(data) = from_str(&text) as std::result::Result<Value, _> {
                if let Value::String(packet_type) = &data["type"] {
                    if packet_type == "authenticate" {
                        if self.user_id.is_some() {
                            self.sender.send(
                                json!({
                                    "type": "authenticate",
                                    "success": false,
                                    "error": "Already authenticated!"
                                })
                                .to_string(),
                            )
                        } else if let Value::String(token) = &data["token"] {
                            let mut state = unsafe { state::DATA.get().unwrap().write().unwrap() };

                            match state.try_authenticate(self.id.clone(), token.to_string()) {
                                StateResult::Success(user_id) => {
                                    let user = crate::database::user::fetch_user(&user_id)
                                        .unwrap()
                                        .unwrap();
                                    self.user_id = Some(user_id);

                                    self.sender.send(
                                        json!({
                                            "type": "authenticate",
                                            "success": true,
                                        })
                                        .to_string(),
                                    )?;

                                    if let Ok(payload) = user.create_payload() {
                                        self.sender.send(
                                            json!({
                                                "type": "ready",
                                                "data": payload
                                            })
                                            .to_string(),
                                        )
                                    } else {
                                        // ! TODO: FIXME: ALL THE NOTIFICATIONS CODE NEEDS TO BE
                                        // ! RESTRUCTURED, IT IS UTTER GARBAGE. :)))))

                                        Ok(())
                                    }
                                }
                                StateResult::DatabaseError => self.sender.send(
                                    json!({
                                        "type": "authenticate",
                                        "success": false,
                                        "error": "Had database error."
                                    })
                                    .to_string(),
                                ),
                                StateResult::InvalidToken => self.sender.send(
                                    json!({
                                        "type": "authenticate",
                                        "success": false,
                                        "error": "Invalid token."
                                    })
                                    .to_string(),
                                ),
                            }
                        } else {
                            self.sender.send(
                                json!({
                                    "type": "authenticate",
                                    "success": false,
                                    "error": "Token not present."
                                })
                                .to_string(),
                            )
                        }
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        unsafe {
            state::DATA
                .get()
                .unwrap()
                .write()
                .unwrap()
                .disconnect(self.user_id.clone(), self.id.clone());
        }

        println!("User disconnected. [{}]", self.id);
    }

    fn on_error(&mut self, err: Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

pub fn launch_server() {
    state::init();

    listen(WS_HOST.to_string(), |sender| Server {
        sender,
        user_id: None,
        id: Ulid::new().to_string(),
    })
    .unwrap()
}
