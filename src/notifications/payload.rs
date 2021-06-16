use std::collections::HashSet;

use crate::{database::*, notifications::events::ClientboundNotification};
use crate::{
    database::{entities::User, get_collection},
    util::result::{Error, Result},
};
use futures::StreamExt;
use mongodb::bson::{doc, from_document};

pub async fn generate_ready(mut user: User) -> Result<ClientboundNotification> {
    let mut user_ids: HashSet<String> = HashSet::new();

    if let Some(relationships) = &user.relations {
        user_ids.extend(
            relationships
                .iter()
                .map(|relationship| relationship.id.clone()),
        );
    }

    let server_ids = user.fetch_server_ids().await?;
    let mut cursor = get_collection("servers")
        .find(
            doc! {
                "_id": {
                    "$in": server_ids
                }
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "servers",
        })?;

    let mut servers = vec![];
    let mut channel_ids = vec![];
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            let server: Server = from_document(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "server",
            })?;

            channel_ids.extend(server.channels.iter().cloned());
            servers.push(server);
        }
    }

    let mut cursor = get_collection("channels")
        .find(
            doc! {
                "$or": [
                    {
                        "_id": {
                            "$in": channel_ids
                        }
                    },
                    {
                        "channel_type": "SavedMessages",
                        "user": &user.id
                    },
                    {
                        "channel_type": "DirectMessage",
                        "recipients": &user.id
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
            let channel = from_document(doc).map_err(|_| Error::DatabaseError {
                operation: "from_document",
                with: "channel",
            })?;

            if let Channel::Group { recipients, .. } = &channel {
                user_ids.extend(recipients.iter().cloned());
            } else if let Channel::DirectMessage { recipients, .. } = &channel {
                user_ids.extend(recipients.iter().cloned());
            }

            channels.push(channel);
        }
    }

    user_ids.remove(&user.id);
    let mut users = if user_ids.len() > 0 {
        user.fetch_multiple_users(user_ids.into_iter().collect::<Vec<String>>())
            .await?
    } else {
        vec![]
    };

    user.relationship = Some(RelationshipStatus::User);
    user.online = Some(true);
    users.push(user);

    Ok(ClientboundNotification::Ready {
        users,
        servers,
        channels
    })
}
