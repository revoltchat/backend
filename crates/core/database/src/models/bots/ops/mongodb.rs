use revolt_database::MongoDb;

use super::AbstractBots;

mod init;
mod scripts;

#[async_trait]
impl AbstractBots for MongoDb {}
