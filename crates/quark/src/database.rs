use std::env;
use std::ops::Deref;

use crate::r#impl::dummy::DummyDb;
use crate::r#impl::mongo::MongoDb;
use crate::AbstractDatabase;

/// Database information to use to create a client
pub enum DatabaseInfo {
    /// Auto-detect the database in use
    Auto,
    /// Use the mock database
    Dummy,
    /// Connect to MongoDB
    MongoDb(String),
    /// Use existing MongoDB connection
    MongoDbFromClient(mongodb::Client),
}

/// Database
#[derive(Debug, Clone)]
pub enum Database {
    /// Mock database
    Dummy(DummyDb),
    /// MongoDB database
    MongoDb(MongoDb),
}

impl DatabaseInfo {
    /// Create a database client from the given database information
    #[async_recursion]
    pub async fn connect(self) -> Result<Database, String> {
        Ok(match self {
            DatabaseInfo::Auto => {
                if let Ok(uri) = env::var("MONGODB") {
                    return DatabaseInfo::MongoDb(uri).connect().await;
                }

                DatabaseInfo::Dummy.connect().await?
            }
            DatabaseInfo::Dummy => Database::Dummy(DummyDb),
            DatabaseInfo::MongoDb(uri) => {
                let client = mongodb::Client::with_uri_str(uri)
                    .await
                    .map_err(|_| "Failed to init db connection.".to_string())?;

                Database::MongoDb(MongoDb(client))
            }
            DatabaseInfo::MongoDbFromClient(client) => Database::MongoDb(MongoDb(client)),
        })
    }
}

impl Deref for Database {
    type Target = dyn AbstractDatabase;

    fn deref(&self) -> &Self::Target {
        match self {
            Database::Dummy(dummy) => dummy,
            Database::MongoDb(mongo) => mongo,
        }
    }
}
