use futures::future::join_all;
use revolt_database::{Database, User};
use revolt_models::v0::OwnedBotsResponse;
use revolt_result::Result;
use rocket::serde::json::Json;
use rocket::State;

/// # Fetch Owned Bots
///
/// Fetch all of the bots that you have control over.
#[openapi(tag = "Bots")]
#[get("/@me")]
pub async fn fetch_owned_bots(db: &State<Database>, user: User) -> Result<Json<OwnedBotsResponse>> {
    let mut bots = db.fetch_bots_by_user(&user.id).await?;
    let user_ids = bots
        .iter()
        .map(|x| x.id.to_owned())
        .collect::<Vec<String>>();

    let mut users = db.fetch_users(&user_ids).await?;

    // Ensure the lists match up exactly.
    bots.sort_by(|a, b| a.id.cmp(&b.id));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(Json(OwnedBotsResponse {
        users: join_all(users.into_iter().map(|user| user.into_self(false))).await,
        bots: bots.into_iter().map(|bot| bot.into()).collect(),
    }))
}

#[cfg(test)]
mod test {
    use crate::{rocket, util::test::TestHarness};
    use revolt_database::Bot;
    use revolt_models::v0;
    use rocket::http::{Header, Status};

    #[rocket::async_test]
    async fn fetch_owned() {
        let harness = TestHarness::new().await;
        let (_, session, user) = harness.new_user().await;

        let (bot, _) = Bot::create(&harness.db, TestHarness::rand_string(), &user, None)
            .await
            .expect("`Bot`");

        let response = harness
            .client
            .get("/bots/@me")
            .header(Header::new("x-session-token", session.token.to_string()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let resp: v0::OwnedBotsResponse = response.into_json().await.expect("`Vec<Bot>`");
        assert_eq!(resp.bots.len(), 1);
        assert_eq!(resp.users.len(), 1);
        assert_eq!(resp.bots[0], bot.into());
        assert_eq!(resp.bots[0].id, resp.users[0].id);
    }
}
