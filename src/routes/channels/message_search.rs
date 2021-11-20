use std::collections::HashSet;

use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};
use rocket::serde::json::{Json, Value};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, FromFormField)]
pub enum Sort {
    Relevance,
    Latest,
    Oldest,
}

impl Default for Sort {
    fn default() -> Sort {
        Sort::Relevance
    }
}

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(length(min = 1, max = 64))]
    query: String,

    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    #[serde(default = "Sort::default")]
    sort: Sort,
    include_users: Option<bool>,
}

#[post("/<target>/search", data = "<options>")]
pub async fn req(user: User, target: Ref, options: Json<Options>) -> Result<Value> {
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let target = target.fetch_channel().await?;
    target.has_messaging()?;

    let perm = permissions::PermissionCalculator::new(&user)
        .with_channel(&target)
        .for_channel()
        .await?;
    if !perm.get_view() {
        Err(Error::MissingPermission)?
    }

    let mut messages = vec![];
    let limit = options.limit.unwrap_or(50);

    let mut filter = doc! {
        "channel": target.id(),
        "$text": {
            "$search": &options.query
        }
    };

    if let Some(doc) = match (&options.before, &options.after) {
        (Some(before), Some(after)) => Some(doc! {
            "lt": before,
            "gt": after
        }),
        (Some(before), _) => Some(doc! {
            "lt": before
        }),
        (_, Some(after)) => Some(doc! {
            "gt": after
        }),
        _ => None
    } {
        filter.insert("_id", doc);
    }

    let mut cursor = get_collection("messages")
        .find(
            filter,
            FindOptions::builder()
                .projection(
                    if let Sort::Relevance = &options.sort {
                        doc! {
                            "score": {
                                "$meta": "textScore"
                            }
                        }
                    } else {
                        doc! {}
                    }
                )
                .limit(limit)
                .sort(
                    match &options.sort {
                        Sort::Relevance => doc! {
                            "score": {
                                "$meta": "textScore"
                            }
                        },
                        Sort::Latest => doc! {
                            "_id": -1
                        },
                        Sort::Oldest => doc! {
                            "_id": 1
                        }
                    }
                )
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "messages",
        })?;
    
    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            messages.push(
                from_document::<Message>(doc).map_err(|_| Error::DatabaseError {
                    operation: "from_document",
                    with: "message",
                })?,
            );
        }
    }

    if options.include_users.unwrap_or_else(|| false) {
        let mut ids = HashSet::new();
        for message in &messages {
            message.add_associated_user_ids(&mut ids);
        }

        ids.remove(&user.id);
        let user_ids = ids.into_iter().collect();
        let users = user.fetch_multiple_users(user_ids).await?;

        if let Channel::TextChannel { server, .. } = target {
            Ok(json!({
                "messages": messages,
                "users": users,
                "members": Server::fetch_members(&server).await?
            }))
        } else {
            Ok(json!({
                "messages": messages,
                "users": users,
            }))
        }
    } else {
        Ok(json!(messages))
    }
}
