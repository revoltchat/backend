//! Fetch direct messages that the current user is involved in
//!
//! This includes group DMs and "active" 1:1 DMs.

use revolt_quark::{
    models::{Channel, User},
    Database, Result,
};

use mongodb::bson::doc;
use rocket::{serde::json::Json, State};

#[get("/dms")]
pub async fn req(db: &State<Database>, user: User) -> Result<Json<Vec<Channel>>> {
    db.find_direct_messages(&user.id).await.map(Json)
}
