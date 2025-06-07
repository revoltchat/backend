use async_std::channel::{unbounded, Receiver, Sender};
use authifier::AuthifierEvent;
use once_cell::sync::Lazy;

use crate::events::client::EventV1;

static Q: Lazy<(Sender<AuthifierEvent>, Receiver<AuthifierEvent>)> = Lazy::new(unbounded);

/// Get sender
pub fn sender() -> Sender<AuthifierEvent> {
    Q.0.clone()
}

/// Start a new worker
pub async fn worker() {
    loop {
        let event = Q.1.recv().await.unwrap();
        match &event {
            AuthifierEvent::CreateSession { .. } | AuthifierEvent::CreateAccount { .. } => {
                EventV1::Auth(event).global().await
            }
            AuthifierEvent::DeleteSession { user_id, .. }
            | AuthifierEvent::DeleteAllSessions { user_id, .. } => {
                let id = user_id.to_string();
                EventV1::Auth(event).private(id).await
            }
        }
    }
}
