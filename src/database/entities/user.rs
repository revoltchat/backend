use crate::database::guards::reference::Ref;
use crate::{
    database::get_collection,
    notifications::events::ClientboundNotification,
    util::result::{Error, Result},
};
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_bson, Bson},
    options::FindOptions,
};
use rauth::auth::Session;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RelationshipStatus {
    None,
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relationship {
    #[serde(rename = "_id")]
    pub id: String,
    pub status: RelationshipStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,

    // ? This should never be pushed to the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = rauth::util::Error;

    async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let session: Session = try_outcome!(request.guard::<Session>().await);

        if let Ok(result) = get_collection("users")
            .find_one(
                doc! {
                    "_id": &session.user_id
                },
                None,
            )
            .await
        {
            if let Some(doc) = result {
                Outcome::Success(from_bson(Bson::Document(doc)).unwrap())
            } else {
                Outcome::Failure((Status::Forbidden, rauth::util::Error::InvalidSession))
            }
        } else {
            Outcome::Failure((
                Status::InternalServerError,
                rauth::util::Error::DatabaseError,
            ))
        }
    }
}

impl User {
    pub fn as_ref(&self) -> Ref {
        Ref {
            id: self.id.to_string(),
        }
    }

    pub async fn generate_ready_payload(self) -> Result<ClientboundNotification> {
        let mut users = vec![];

        if let Some(relationships) = &self.relations {
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

        users.push(self);

        Ok(ClientboundNotification::Ready { users })
    }
}
