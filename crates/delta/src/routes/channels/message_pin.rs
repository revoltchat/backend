use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference}, Database, PartialMessage, User
};
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Pins a message
///
/// Pins a message by its id.
#[openapi(tag = "Messaging")]
#[post("/<target>/messages/<msg>/pin")]
pub async fn message_pin(
    db: &State<Database>,
    user: User,
    target: Reference,
    msg: Reference,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageMessages)?;

    let mut message = msg.as_message_in_channel(db, &channel.id()).await?;

    if message.pinned.unwrap_or_default() {
        return Err(create_error!(AlreadyPinned))
    }

    message.update(db, PartialMessage {
        pinned: Some(true),
        ..Default::default()
    }, vec![]).await?;

    Ok(EmptyResponse)
}

#[cfg(test)]
mod test {
    use crate::{rocket, util::test::TestHarness};
    use revolt_database::{events::client::EventV1, util::{idempotency::IdempotencyKey, reference::Reference}, Member, Message, Server};
    use revolt_models::v0;
    use rocket::http::{Header, Status};

    #[rocket::async_test]
    async fn pin_message() {
        let mut harness = TestHarness::new().await;
        let (_, session, user) = harness.new_user().await;

        let (server, channels) = Server::create(
            &harness.db,
            v0::DataCreateServer {
                name: "Test Server".to_string(),
                ..Default::default()
            },
            &user,
            true
        ).await.expect("Failed to create test server");

        let (member, channels) = Member::create(&harness.db, &server, &user, Some(channels)).await.expect("Failed to create member");
        let channel = &channels[0];

        let message = Message::create_from_api(
            &harness.db,
            channel.clone(),
            v0::DataMessageSend {
                content:Some("Test message".to_string()),
                nonce: None,
                attachments: None,
                replies: None,
                embeds: None,
                masquerade: None,
                interactions: None,
                flags: None
            },
            v0::MessageAuthor::User(&user.clone().into(&harness.db, Some(&user)).await),
            Some(user.clone().into(&harness.db, Some(&user)).await),
            Some(member.into()),
            user.limits().await,
            IdempotencyKey::unchecked_from_string("0".to_string()),
            false,
            false
        )
        .await
        .expect("Failed to create message");

        let response = harness
            .client
            .post(format!("/channels/{}/messages/{}/pin", channel.id(), &message.id))
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
        drop(response);

        harness.wait_for_event(&channel.id(), |event| {
            match event {
                EventV1::MessageUpdate { id, channel: channel_id, data, .. } => {
                    assert_eq!(id, &message.id);
                    assert_eq!(channel_id, &channel.id());
                    assert_eq!(data.pinned, Some(true));

                    true
                },
                _ => false
            }
        }).await;

        let updated_message = Reference::from_unchecked(message.id)
            .as_message(&harness.db)
            .await
            .expect("Failed to find updated message");

        assert_eq!(updated_message.pinned, Some(true));
    }
}
