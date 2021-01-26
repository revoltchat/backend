use std::collections::HashSet;

use crate::{database::*, notifications::events::ClientboundNotification};
use crate::{
    database::{entities::User, get_collection},
    util::result::{Error, Result},
};
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};

pub async fn generate_ready(mut user: User) -> Result<ClientboundNotification> {
    let mut users = vec![];
    let mut user_ids: HashSet<String> = HashSet::new();

    if let Some(relationships) = &user.relations {
        user_ids.extend(
            relationships
                .iter()
                .map(|relationship| relationship.id.clone()),
        );
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
            }

            channels.push(channel);
        }
    }

    if user_ids.len() > 0 {
        let mut cursor = get_collection("users")
            .find(
                doc! {
                    "_id": {
                        "$in": user_ids.into_iter().collect::<Vec<String>>()
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
                let mut other: User = from_document(doc).map_err(|_| Error::DatabaseError {
                    operation: "from_document",
                    with: "user",
                })?;

                if let Some(relationships) = &user.relations {
                    other.relationship = Some(
                        if let Some(relationship) = relationships.iter().find(|x| other.id == x.id)
                        {
                            relationship.status.clone()
                        } else {
                            RelationshipStatus::None
                        },
                    );
                }

                let permissions = PermissionCalculator::new(&user)
                    .with_mutual_connection()
                    .with_user(&other)
                    .for_user_given()
                    .await?;

                users.push(other.with(permissions));
            }
        }
    }

    user.online = Some(true);
    users.push(user);

    Ok(ClientboundNotification::Ready { users, channels })
}
