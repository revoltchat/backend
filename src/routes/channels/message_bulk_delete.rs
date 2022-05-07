use chrono::Utc;
use revolt_quark::{
    models::{Message, User},
    perms, Db, EmptyResponse, Error, Permission, Ref, Result,
};
use rocket::serde::json::Json;
use serde::Deserialize;
use validator::Validate;

/// # Search Parameters
#[derive(Validate, Deserialize, JsonSchema)]
pub struct OptionsBulkDelete {
    /// Message IDs
    #[validate(length(min = 1, max = 100))]
    ids: Vec<String>,
}

/// # Bulk Delete Messages
///
/// Delete multiple messages you've sent or one you have permission to delete.
///
/// This will always require `ManageMessages` permission regardless of whether you own the message or not.
///
/// Messages must have been sent within the past 1 week.
#[openapi(tag = "Messaging")]
#[delete("/<target>/messages/bulk", data = "<options>", rank = 1)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: Json<OptionsBulkDelete>,
) -> Result<EmptyResponse> {
    let options = options.into_inner();
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    for id in &options.ids {
        if ulid::Ulid::from_string(id)
            .map_err(|_| Error::InvalidOperation)?
            .datetime()
            .signed_duration_since(Utc::now())
            .num_days()
            .abs()
            > 7
        {
            return Err(Error::InvalidOperation);
        }
    }

    perms(&user)
        .channel(&target.as_channel(db).await?)
        .throw_permission(db, Permission::ManageMessages)
        .await?;

    Message::bulk_delete(db, &target.id, options.ids)
        .await
        .map(|_| EmptyResponse)
}
