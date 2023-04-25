mod mongodb;
mod reference;

pub use self::mongodb::*;
pub use self::reference::*;

/// Database information to use to create a client
pub enum DatabaseInfo {
    /// Auto-detect the database in use
    Auto,
    /// Auto-detect the database in use and create an empty testing database
    Test(String),
    /// Use the mock database
    Reference,
    /// Connect to MongoDB
    MongoDb { uri: String, database_name: String },
    /// Use existing MongoDB connection
    MongoDbFromClient(::mongodb::Client, String),
}

/// Database
#[derive(Clone)]
pub enum Database {
    /// Mock database
    Reference(ReferenceDb),
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
                    return DatabaseInfo::MongoDb {
                        uri,
                        database_name: "revolt".to_string(),
                    }
                    .connect()
                    .await;
                }

                DatabaseInfo::Reference.connect().await?
            }
            DatabaseInfo::Test(database_name) => {
                if let Ok(uri) = std::env::var("MONGODB") {
                    return DatabaseInfo::MongoDb { uri, database_name }.connect().await;
                }

                DatabaseInfo::Reference.connect().await?
            }
            DatabaseInfo::Reference => Database::Reference(Default::default()),
            DatabaseInfo::MongoDb { uri, database_name } => {
                let client = ::mongodb::Client::with_uri_str(uri)
                    .await
                    .map_err(|_| "Failed to init db connection.".to_string())?;

                Database::MongoDb(MongoDb(client, database_name))
            }
            DatabaseInfo::MongoDbFromClient(client, database_name) => {
                Database::MongoDb(MongoDb(client, database_name))
            }
        })
    }
}
