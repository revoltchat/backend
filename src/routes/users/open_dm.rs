use crate::database::entities::{Channel, User};
use crate::database::guards::reference::Ref;
use crate::util::result::{Error, Result};
use crate::database::get_collection;
use rocket_contrib::json::JsonValue;
use mongodb::bson::doc;
use ulid::Ulid;

#[get("/<target>/dm")]
pub async fn req(user: User, target: Ref) -> Result<JsonValue> {
    let query = if user.id == target.id {
        doc! {
            "type": "SavedMessages",
            "user": &user.id
        }
    } else {
        doc! {
            "type": "DirectMessage",
            "recipients": {
                "$all": [ &user.id, &target.id ]
            }
        }
    };

    let existing_channel = get_collection("channels")
        .find_one(query, None)
        .await
        .map_err(|_| Error::DatabaseError { operation: "find_one", with: "channel" })?;

    if let Some(doc) = existing_channel {
        Ok(json!(doc))
    } else {
        let id = Ulid::new().to_string();
        let channel = if user.id == target.id {
            Channel::SavedMessages {
                id,
                user: user.id
            }
        } else {
            Channel::DirectMessage {
                id,
                active: false,
                recipients: vec! [
                    user.id,
                    target.id
                ]
            }
        };

        channel.save().await?;
        Ok(json!(channel))
    }
}
