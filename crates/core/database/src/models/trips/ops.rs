use async_trait::async_trait;
use chrono::{DateTime, Utc};
use revolt_result::Result;
use crate::models::trips::model::Trip;

#[async_trait]
pub trait AbstractTrips: Sync + Send {
    async fn insert_trip(&self, trip: &Trip) -> Result<()>;
    async fn fetch_trips_by_date_and_destination(
        &self,
        date: DateTime<Utc>,
        destination: &str,
    ) -> Result<Vec<Trip>>;
}
