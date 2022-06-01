use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::models::{Message, User};
use crate::variables::delta::{APP_URL, AUTUMN_URL, PUBLIC_URL};

/// Push Notification
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushNotification {
    /// Known author name
    pub author: String,
    /// URL to author avatar
    pub icon: String,
    /// URL to first matching attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Message content or system message information
    pub body: String,
    /// Unique tag, usually the channel ID
    pub tag: String,
    /// Timestamp at which this notification was created
    pub timestamp: u64,
    /// URL to open when clicking notification
    pub url: String,
}

impl PushNotification {
    /// Create a new notification from a given message, author and channel ID
    pub fn new(msg: Message, author: Option<&User>, channel_id: &str) -> Self {
        let icon = if let Some(author) = author {
            if let Some(avatar) = &author.avatar {
                format!("{}/avatars/{}", &*AUTUMN_URL, avatar.id)
            } else {
                format!("{}/users/{}/default_avatar", &*PUBLIC_URL, msg.author)
            }
        } else {
            format!("{}/assets/logo.png", &*APP_URL)
        };

        let image = msg.attachments.and_then(|attachments| {
            attachments
                .first()
                .map(|v| format!("{}/attachments/{}", &*AUTUMN_URL, v.id))
        });

        let body = if let Some(sys) = msg.system {
            sys.into()
        } else if let Some(text) = msg.content {
            text
        } else {
            "Empty Message".to_string()
        };

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            author: author
                .map(|x| x.username.to_string())
                .unwrap_or_else(|| "Revolt".to_string()),
            icon,
            image,
            body,
            tag: channel_id.to_string(),
            timestamp,
            url: format!("{}/channel/{}/{}", &*APP_URL, channel_id, msg.id),
        }
    }
}
