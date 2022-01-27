use revolt_quark::{Error, Result};

use futures::try_join;
use mongodb::bson::doc;
use mongodb::options::{Collation, FindOneOptions};
use rocket::serde::json::Value;

#[put("/<username>/friend")]
pub async fn req(/*user: UserRef,*/ username: String) -> Result<Value> {
    todo!()
}
