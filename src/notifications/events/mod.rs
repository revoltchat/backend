use serde::{Deserialize, Serialize};

pub mod message;

#[derive(Serialize, Deserialize, Debug)]
pub enum Notification {
    MessageCreate(message::Create),
}
