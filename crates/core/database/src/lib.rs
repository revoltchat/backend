#[macro_use]
extern crate serde;

#[macro_use]
extern crate async_recursion;

macro_rules! database_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Clone)]
            $item
        )+
    };
}

#[cfg(feature = "mongodb")]
pub use mongodb;

mod drivers;
pub use drivers::*;

/// Database information to use to create a client
pub enum DatabaseInfo {
    /// Auto-detect the database in use
    Auto,
    /// Use the mock database
    Dummy,
    /// Connect to MongoDB
    MongoDb(String),
    /// Use existing MongoDB connection
    MongoDbFromClient(::mongodb::Client),
}

/// Database
#[derive(Clone)]
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
                if let Ok(uri) = std::env::var("MONGODB") {
                    return DatabaseInfo::MongoDb(uri).connect().await;
                }

                DatabaseInfo::Dummy.connect().await?
            }
            DatabaseInfo::Dummy => Database::Dummy(DummyDb {}),
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
