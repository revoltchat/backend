#[macro_use]
extern crate serde;

#[macro_use]
extern crate async_recursion;

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate log;

#[macro_use]
extern crate revolt_optional_struct;

#[macro_use]
extern crate revolt_result;

#[cfg(feature = "mongodb")]
pub use ::mongodb;  
//pub use drivers::mongodb;

use futures::stream::StreamExt; 

#[cfg(feature = "mongodb")]
#[macro_use]
extern crate bson;

macro_rules! database_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Clone)]
            $item
        )+
    };
}

macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
            $item
        )+
    };
}

macro_rules! auto_derived_partial {
    ( $item:item, $name:expr ) => {
        #[derive(OptionalStruct, Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
        #[optional_derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
        #[optional_name = $name]
        #[opt_skip_serializing_none]
        #[opt_some_priority]
        $item
    };
}

// ----------------------------------------------------------------------------
// Modules
// ----------------------------------------------------------------------------

pub mod drivers;
pub use drivers::*;

#[cfg(test)]
macro_rules! database_test {
    ( | $db: ident | $test:expr ) => {
        let db = $crate::DatabaseInfo::Test(format!(
            "{}:{}",
            file!().replace('/', "_").replace(".rs", ""),
            line!()
        ))
        .connect()
        .await
        .expect("Database connection failed.");

        db.drop_database().await;

        #[allow(clippy::redundant_closure_call)]
        (|$db: $crate::Database| $test)(db.clone()).await;

        db.drop_database().await
    };
}

mod models;
pub mod util;
pub use models::*;

pub mod events;

// ----------------------------------------------------------------------------
// Utility
// ----------------------------------------------------------------------------

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

// ----------------------------------------------------------------------------
// DatabaseTrait setup
// ----------------------------------------------------------------------------

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use revolt_result::{Result, create_database_error};

use crate::models::trips::model::Trip;
pub use crate::drivers::Database;
pub use drivers::mongodb::MongoDb;

use bson::{doc, DateTime as BsonDateTime};

/// A trait exposing the DB methods your code calls
#[async_trait]
pub trait DatabaseTrait: Sync + Send {
    async fn insert_trip(&self, trip: &Trip) -> Result<()>;
    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
    ) -> Result<Vec<Trip>>;
}

// ----------------------------------------------------------------------------
// Implementation for MongoDb
// ----------------------------------------------------------------------------

#[async_trait]
impl DatabaseTrait for MongoDb {
    async fn insert_trip(&self, trip: &Trip) -> Result<()> {
        // Log the entire Trip struct
        eprintln!("[MongoDb::insert_trip] Inserting trip = {:?}", trip);

        let collection = self.col::<Trip>("trips");
        match collection.insert_one(trip, None).await {
            Ok(_res) => {
                eprintln!("[MongoDb::insert_trip] Insert SUCCESS");
                Ok(())
            }
            Err(err) => {
                eprintln!("[MongoDb::insert_trip] Insert ERROR: {}", err);
                Err(create_database_error!("insert", "trips"))
            }
        }
    }

    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
    ) -> Result<Vec<Trip>> {
        eprintln!("[MongoDb::fetch_trips_by_date_and_destination] date = {}, destination = {}",
                  date, destination);

        // Convert Chrono -> BSON
        let mongo_date = BsonDateTime::from_chrono(date);
        eprintln!("[MongoDb] Converted date to BsonDateTime = {:?}", mongo_date);

        let filter = doc! {
            "destination": destination,
            "start_date": { "$lte": mongo_date },
            "end_date":   { "$gte": mongo_date }
        };
        eprintln!("[MongoDb] Using filter = {:?}", filter);

        let collection = self.col::<Trip>("trips");
        let mut cursor = match collection.find(filter, None).await {
            Ok(cur) => {
                eprintln!("[MongoDb] find() SUCCESS, got a cursor. Iterating docs...");
                cur
            }
            Err(err) => {
                eprintln!("[MongoDb] find() ERROR: {}", err);
                return Err(create_database_error!("find", "trips"));
            }
        };

        let mut trips = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(trip) => {
                    eprintln!("[MongoDb] Found doc => {:?}", trip);
                    trips.push(trip);
                }
                Err(err) => {
                    eprintln!("[MongoDb] Cursor read error: {}", err);
                }
            }
        }

        eprintln!("[MongoDb] Found {} trip(s) matching date={} destination={}",
                  trips.len(), date, destination);

        Ok(trips)
    }
}

// ----------------------------------------------------------------------------
// Implementation for enum Database
// ----------------------------------------------------------------------------

#[async_trait]
impl DatabaseTrait for Database {
    async fn insert_trip(&self, trip: &Trip) -> Result<()> {
        match self {
            Database::MongoDb(mongo) => {
                eprintln!("[Database::insert_trip -> MongoDb] delegating...");
                mongo.insert_trip(trip).await
            }
            Database::Reference(_mock) => {
                // If you have a mock/Reference variant, either implement or unimplemented!()
                eprintln!("[Database::insert_trip -> Reference] unimplemented");
                unimplemented!("Reference DB not implemented for insert_trip.")
            }
        }
    }

    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
    ) -> Result<Vec<Trip>> {
        match self {
            Database::MongoDb(mongo) => {
                eprintln!("[Database::fetch_trips -> MongoDb] delegating...");
                mongo.fetch_trips_by_date_and_destination(date, destination).await
            }
            Database::Reference(_mock) => {
                eprintln!("[Database::fetch_trips -> Reference] unimplemented");
                unimplemented!("Reference DB not implemented for fetch_trips_by_date_and_destination.")
            }
        }
    }
}
