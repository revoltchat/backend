use revolt_quark::{
    models::{Channel, User},
    Db, EmptyResponse, Error, Permission, Ref, Result,
};

/// # Remove Member from Group
///
/// Removes a user from the group.
#[openapi(tag = "Groups")]
#[delete("/<target>/recipients/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;

    match &channel {
        Channel::Group {
            owner, recipients, ..
        } => {
            if &user.id != owner {
                return Error::from_permission(Permission::ManageChannel);
            }

            let member = member.as_user(db).await?;
            if user.id == member.id {
                return Err(Error::CannotRemoveYourself);
            }

            if !recipients.iter().any(|x| *x == member.id) {
                return Err(Error::NotInGroup);
            }

            channel
                .remove_user_from_group(db, &member.id, Some(&user.id), false)
                .await
                .map(|_| EmptyResponse)
        }
        _ => Err(Error::InvalidOperation),
    }
}

#[cfg(test)]
mod test {
    use crate::{rocket, util::test::TestHarness};
    use revolt_database::{events::client::EventV1, Channel, RelationshipStatus};
    use revolt_models::v0;
    use rocket::http::{Header, Status};

    #[rocket::async_test]
    async fn success_remove_member() {
        let mut harness = TestHarness::new().await;
        let (_, session, mut user) = harness.new_user().await;
        let (_, _, mut other_user) = harness.new_user().await;

        #[allow(clippy::disallowed_methods)]
        user.apply_relationship(
            &harness.db,
            &mut other_user,
            RelationshipStatus::Friend,
            RelationshipStatus::Friend,
        )
        .await
        .unwrap();

        let group = Channel::create_group(
            &harness.db,
            v0::DataCreateGroup {
                name: TestHarness::rand_string(),
                ..Default::default()
            },
            user.id.to_string(),
        )
        .await
        .unwrap();

        let response = harness
            .client
            .put(format!(
                "/channels/{}/recipients/{}",
                group.id(),
                other_user.id
            ))
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
        drop(response);

        harness
            .wait_for_event(&format!("{}!", other_user.id), |event| match event {
                EventV1::ChannelCreate(channel) => channel.id() == group.id(),
                _ => false,
            })
            .await;

        let event = harness
            .wait_for_event(&group.id(), |event| match event {
                EventV1::ChannelGroupJoin { id, .. } => id == &group.id(),
                _ => false,
            })
            .await;

        match event {
            EventV1::ChannelGroupJoin { user, .. } => assert_eq!(user, other_user.id),
            _ => unreachable!(),
        };

        let message = harness.wait_for_message(&group.id()).await;

        assert_eq!(
            message.system,
            Some(v0::SystemMessage::UserAdded {
                id: other_user.id.to_string(),
                by: user.id.to_string()
            })
        );
    }

    #[rocket::async_test]
    async fn fail_not_in_group() {
        let harness = TestHarness::new().await;
        let (_, session, user) = harness.new_user().await;
        let (_, _, other_user) = harness.new_user().await;

        let group = Channel::create_group(
            &harness.db,
            v0::DataCreateGroup {
                name: TestHarness::rand_string(),
                ..Default::default()
            },
            user.id.to_string(),
        )
        .await
        .unwrap();

        let response = harness
            .client
            .delete(format!(
                "/channels/{}/recipients/{}",
                group.id(),
                other_user.id
            ))
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        dbg!(response.into_string().await);
        // assert_eq!(response.status(), Status::NotFound);
    }

    #[rocket::async_test]
    async fn fail_not_group_owner() {
        let harness = TestHarness::new().await;
        let (_, _, user) = harness.new_user().await;
        let (_, session, other_user) = harness.new_user().await;
        let (_, _, user_to_be_kicked) = harness.new_user().await;

        let group = Channel::create_group(
            &harness.db,
            v0::DataCreateGroup {
                name: TestHarness::rand_string(),
                users: vec![&other_user.id, &user_to_be_kicked.id]
                    .into_iter()
                    .cloned()
                    .collect(),
                ..Default::default()
            },
            user.id.to_string(),
        )
        .await
        .unwrap();

        let response = harness
            .client
            .delete(format!(
                "/channels/{}/recipients/{}",
                group.id(),
                user_to_be_kicked.id
            ))
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Forbidden);
    }
}
