use revolt_database::{util::reference::Reference, voice::{delete_voice_state, get_channel_node, get_user_voice_channels, VoiceClient}, Database, User};
use revolt_result::{create_error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

/// # Delete Bot
///
/// Delete a bot by its id.
#[openapi(tag = "Bots")]
#[delete("/<target>")]
pub async fn delete_bot(
    db: &State<Database>,
    voice_client: &State<VoiceClient>,
    user: User,
    target: Reference<'_>,
) -> Result<EmptyResponse> {
    let bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(create_error!(NotFound));
    }

    bot.delete(db).await?;

    for channel_id in get_user_voice_channels(&bot.id).await? {
        let node = get_channel_node(&channel_id).await?.unwrap();
        let channel = Reference::from_unchecked(&channel_id).as_channel(db).await?;

        voice_client.remove_user(&node, &bot.id, &channel_id).await?;

        delete_voice_state(&channel_id, channel.server(), &bot.id).await?;
    }

    Ok(EmptyResponse)
}

#[cfg(test)]
mod test {
    use crate::{rocket, util::test::TestHarness};
    use revolt_database::{events::client::EventV1, Bot};
    use rocket::http::{Header, Status};

    #[rocket::async_test]
    async fn delete_bot() {
        let mut harness = TestHarness::new().await;
        let (_, session, user) = harness.new_user().await;

        let (bot, _) = Bot::create(&harness.db, TestHarness::rand_string(), &user, None)
            .await
            .expect("`Bot`");

        let response = harness
            .client
            .delete(format!("/bots/{}", bot.id))
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
        assert!(harness.db.fetch_bot(&bot.id).await.is_err());
        drop(response);

        let event = harness
            .wait_for_event(&bot.id, |event| match event {
                EventV1::UserUpdate { id, .. } => id == &bot.id,
                _ => false,
            })
            .await;

        match event {
            EventV1::UserUpdate { data, .. } => {
                assert_eq!(data.flags, Some(2));
            }
            _ => unreachable!(),
        }
    }
}
