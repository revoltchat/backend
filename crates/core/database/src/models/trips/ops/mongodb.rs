use async_trait::async_trait;
use bson::doc;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use revolt_result::{Result, create_database_error};
use crate::models::trips::model::Trip;
use crate::models::trips::ops::AbstractTrips;
use crate::drivers::mongodb::MongoDb;
use bson::BsonDateTime;


#[async_trait]
impl AbstractTrips for MongoDb {
    async fn insert_trip(&self, trip: &Trip) -> Result<()> {
        let collection = self.col::<Trip>("trips");
        collection
            .insert_one(trip, None)
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("insert", "trips"))
    }

    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
    ) -> Result<Vec<Trip>> {
        let collection = self.col::<Trip>("trips");
        println!("Using date = {:?}, destination = {:?}", date, destination);
        let mongo_date = bson::DateTime::from_chrono(date);
        let filter = doc! {
            "destination": destination,
            "start_date": { "$lte": mongo_date },
            "end_date": { "$gte": mongo_date }
        };
        println!("Filter = {:?}", filter);
                        
        let mut cursor = collection.find(filter, None).await
        .map_err(|e| {
            eprintln!("Find error: {e:?}");
            create_database_error!("find", "trips")
        })?;
    
    let mut trips = Vec::new();
    while let Some(result) = cursor.next().await {
        if let Ok(trip) = result {
            trips.push(trip);
        } else {
            eprintln!("Cursor read error: {result:?}");
        }
    }
    println!("Found {} trips!", trips.len());
    Ok(trips)
        }
}
