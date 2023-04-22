use revolt_database::DummyDb;

use super::AbstractBots;

#[async_trait]
impl AbstractBots for DummyDb {}
