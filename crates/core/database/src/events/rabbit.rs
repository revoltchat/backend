use revolt_models::v0::PushNotification;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MessageSentNotification {
    pub notification: PushNotification,
    pub users: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BasicPayload {
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PayloadKind {
    MessageNotification(PushNotification),
}

#[derive(Serialize, Deserialize)]
pub struct PayloadToService {
    pub notification: PayloadKind,
    pub token: String,
}
