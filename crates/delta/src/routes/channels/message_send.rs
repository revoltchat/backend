use revolt_quark::{
    models::{message::DataMessageSend, Message, User},
    perms,
    types::push::MessageAuthor,
    web::idempotency::IdempotencyKey,
    Db, Error, Permission, Ref, Result,
};

use rocket::serde::json::Json;
use validator::Validate;

/// # Send Message
///
/// Sends a message to the given channel.
#[openapi(tag = "Messaging")]
#[post("/<target>/messages", data = "<data>")]
pub async fn message_send(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataMessageSend>,
    idempotency: IdempotencyKey,
) -> Result<Json<Message>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    // Ensure we have permissions to send a message
    let channel = target.as_channel(db).await?;

    let mut permissions = perms(&user).channel(&channel);
    permissions
        .throw_permission_and_view_channel(db, Permission::SendMessage)
        .await?;

    // Verify permissions for masquerade
    if let Some(masq) = &data.masquerade {
        permissions
            .throw_permission(db, Permission::Masquerade)
            .await?;

        if masq.colour.is_some() {
            permissions
                .throw_permission(db, Permission::ManageRole)
                .await?;
        }
    }

    // Check permissions for embeds
    if data.embeds.as_ref().is_some_and(|v| !v.is_empty()) {
        permissions
            .throw_permission(db, Permission::SendEmbeds)
            .await?;
    }

    // Check permissions for files
    if data.attachments.as_ref().is_some_and(|v| !v.is_empty()) {
        permissions
            .throw_permission(db, Permission::UploadFiles)
            .await?;
    }

    // Ensure interactions information is correct
    if let Some(interactions) = &data.interactions {
        interactions.validate(db, &mut permissions).await?;
    }

    // Create the message
    let message = channel
        .send_message(
            db,
            data,
            MessageAuthor::User(&user),
            idempotency,
            permissions
                .has_permission(db, Permission::SendEmbeds)
                .await?,
        )
        .await?;

    Ok(Json(message))
}
