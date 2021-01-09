use crate::notifications::events::ClientboundNotification;
use crate::{
    database::{entities::User, get_collection},
    util::result::{Error, Result},
};
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_bson, Bson},
    options::FindOptions,
};

pub async fn generate_ready(user: User) -> Result<ClientboundNotification> {
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
                let mut user: User =
                    from_bson(Bson::Document(doc)).map_err(|_| Error::DatabaseError {
                        operation: "from_bson",
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

                users.push(user);
            }
        }
    }

    users.push(user);

    Ok(ClientboundNotification::Ready { users })
}
