use revolt_database::{iso8601_timestamp::{Duration, Timestamp}, util::oauth2::TokenType, Database};
use revolt_result::Result;
use tokio::time::sleep;

pub async fn task(db: Database) -> Result<()> {
    let lifetime = Duration::new(TokenType::Access.lifetime().num_seconds(), 0);

    loop {
        let deauthorized_bots = db.fetch_deauthorized_authorized_bots().await?;

        for bot in deauthorized_bots {
            if let Some(deauthorized_at) = &bot.deauthorized_at {
                if deauthorized_at.saturating_add(lifetime) < Timestamp::now_utc() {
                    db.delete_authorized_bot(&bot.id).await?;
                };
            };
        };

        // 1 hour
        sleep(std::time::Duration::from_secs(60 * 60)).await;
    }
}