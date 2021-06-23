use std::collections::HashSet;

use crate::database::*;
use crate::util::result::{Error, Result};

use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document},
    options::FindOptions,
};
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, FromFormValue)]
pub enum Sort {
    Latest,
    Oldest,
}

#[derive(Validate, Serialize, Deserialize, FromForm)]
pub struct Options {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,
    #[validate(length(min = 26, max = 26))]
    before: Option<String>,
    #[validate(length(min = 26, max = 26))]
    after: Option<String>,
    sort: Option<Sort>,
    include_users: Option<bool>,
}

#[get("/<target>/messages?<options..>")]
pub async fn req(user: User, target: Ref, options: Form<Options>) -> Result<JsonValue> {
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

    let mut query = doc! { "channel": target.id() };

    if let Some(before) = &options.before {
        query.insert("_id", doc! { "$lt": before });
    }

    if let Some(after) = &options.after {
        query.insert("_id", doc! { "$gt": after });
    }

    let sort = if let Sort::Latest = options.sort.as_ref().unwrap_or_else(|| &Sort::Latest) {
        -1
    } else {
        1
    };
    let mut cursor = get_collection("messages")
        .find(
            query,
            FindOptions::builder()
                .limit(options.limit.unwrap_or(50))
                .sort(doc! {
                    "_id": sort
                })
                .build(),
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "find",
            with: "messages",
        })?;

    let mut messages = vec![];
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
            ids.insert(message.author.clone());
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
