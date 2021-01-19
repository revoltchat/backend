use crate::notifications::events::ClientboundNotification;
use crate::{
    database::{entities::User, get_collection},
    util::result::{Error, Result},
};
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};

use super::websocket::is_online;

pub async fn generate_ready(mut user: User) -> Result<ClientboundNotification> {
    let mut users = vec![];

    if let Some(relationships) = &user.relations {
        let user_ids: Vec<String> = relationships
            .iter()
            .map(|relationship| relationship.id.clone())
            .collect();

        let mut cursor = get_collection("users")
            .find(
                doc! {
                    "_id": {
                        "$in": user_ids
                    }
                },
                FindOptions::builder()
                    .projection(doc! { "_id": 1, "username": 1 })
                    .build(),
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: "users",
            })?;

        while let Some(result) = cursor.next().await {
            if let Ok(doc) = result {
                let mut user: User = from_document(doc).map_err(|_| Error::DatabaseError {
                    operation: "from_document",
                    with: "user",
                })?;

                user.relationship = Some(
                    relationships
                        .iter()
                        .find(|x| user.id == x.id)
                        .ok_or_else(|| Error::InternalError)?
                        .status
                        .clone(),
                );

                user.online = Some(is_online(&user.id));

                users.push(user);
            }
        }
    }

    let mut cursor = get_collection("channels")
        .find(
            doc! {
                "$or": [
                    {
                        "channel_type": "SavedMessages",
                        "user": &user.id
                    },
                    {
                        "channel_type": "DirectMessage",
                        "recipients": &user.id,
                        "active": true
                    },
                    {
                        "channel_type": "Group",
                        "recipients": &user.id
                    }
                ]
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "channels",
        })?;

    let mut channels = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            channels.push(from_document(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "channel",
            })?);
        }
    }

    user.online = Some(true);
    users.push(user);

    Ok(ClientboundNotification::Ready { users, channels })
}
