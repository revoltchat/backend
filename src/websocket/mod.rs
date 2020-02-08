extern crate ws;

use crate::database;

use ulid::Ulid;
use std::sync::RwLock;
use hashbrown::HashMap;

use bson::{ bson, doc };
use serde_json::{ Value, from_str, json };

use ws::{ listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error };

struct Cell {
	id: String,
	out: Sender,
}

use once_cell::sync::OnceCell;
static mut CLIENTS: OnceCell<RwLock<HashMap<String, Vec<Cell>>>> = OnceCell::new();

struct Server {
	out: Sender,
	id: Option<String>,
	internal: String,
}

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
		Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
		if let Message::Text(text) = msg {
			let data: Value = from_str(&text).unwrap();

			if let Value::String(packet_type) = &data["type"] {
				match packet_type.as_str() {
					"authenticate" => {
						if let Some(_) = self.id {
							self.out.send(
								json!({
									"error": "Already authenticated!"
								})
								.to_string()
							)
						} else if let Value::String(token) = &data["token"] {
							let col = database::get_collection("users");
							
							match col.find_one(
								doc! { "access_token": token },
								None
							).unwrap() {
								Some(u) => {
									let id = u.get_str("_id").expect("Missing id.");

									unsafe {
										let mut map = CLIENTS.get_mut().unwrap().write().unwrap();
										let cell = Cell { id: self.internal.clone(), out: self.out.clone() };
										if map.contains_key(&id.to_string()) {
											map.get_mut(&id.to_string())
												.unwrap()
												.push(cell);
										} else {
											map.insert(id.to_string(), vec![cell]);
										}
									}

									println!("Websocket client connected. [ID: {} // {}]", id.to_string(), self.internal);

									self.id = Some(id.to_string());
									self.out.send(
										json!({
											"success": true
										})
										.to_string()
									)
								},
								None =>
									self.out.send(
										json!({
											"error": "Invalid authentication token."
										})
										.to_string()
									)
							}
						} else {
							self.out.send(
								json!({
									"error": "Missing authentication token."
								})
								.to_string()
							)
						}
					},
					_ => Ok(())
				}
			} else {
				Ok(())
			}
		} else {
			Ok(())
		}
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            CloseCode::Abnormal => println!(
                "Closing handshake failed! Unable to obtain closing status from client."),
            _ => println!("The client encountered an error: {}", reason),
        }

		if let Some(id) = &self.id {
			println!("Websocket client disconnected. [ID: {} // {}]", id, self.internal);
			unsafe {
				let mut map = CLIENTS.get_mut().unwrap().write().unwrap();
				let arr = map.get_mut(&id.clone()).unwrap();

				if arr.len() == 1 {
					map.remove(&id.clone());
				} else {
					let index = arr.iter().position(|x| x.id == self.internal).unwrap();
					arr.remove(index);
					println!("User [{}] is still connected {} times", self.id.as_ref().unwrap(), arr.len());
				}
			}
		}
    }

    fn on_error(&mut self, err: Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

pub fn launch_server() {
	unsafe {
		if let Err(_) = CLIENTS.set(RwLock::new(HashMap::new())) {
			panic!("Failed to set CLIENTS map!");
		}
	}

	listen("127.0.0.1:3012", |out| { Server { out: out, id: None, internal: Ulid::new().to_string() } }).unwrap()
}

pub fn send_message(id: String, message: String) -> std::result::Result<(), ()> {
	unsafe {
		let map = CLIENTS.get().unwrap().read().unwrap();
		if map.contains_key(&id) {
			let arr = map.get(&id).unwrap();

			for item in arr {
				if let Err(_) = item.out.send(message.clone()) {
					return Err(());
				}
			}
		}

		Ok(())
	}
}

// ! TODO: WRITE THREADED QUEUE SYSTEM
// ! FETCH RECIPIENTS HERE INSTEAD OF IN METHOD

pub fn queue_message(ids: Vec<String>, message: String) {
	for id in ids {
		send_message(id, message.clone()).expect("uhhhhhhhhhh can i get uhhhhhhhhhhhhhhhhhh mcdonald cheese burger with fries");
	}
}
