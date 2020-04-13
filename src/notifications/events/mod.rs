use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub mod message;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Notification {
    MessageCreate(message::Create),
}

impl Notification {
    pub fn serialize(self) -> String {
        if let Value::Object(obj) = json!(self) {
            let (key, value) = obj.iter().next().unwrap();

            if let Value::Object(data) = value {
                let mut data = data.clone();
                data.insert("type".to_string(), Value::String(key.to_string()));
                json!(data).to_string()
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }
}
