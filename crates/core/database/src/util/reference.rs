use revolt_result::Result;
#[cfg(feature = "rocket-impl")]
use rocket::request::FromParam;
#[cfg(feature = "rocket-impl")]
use schemars::{
    schema::{InstanceType, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};

use crate::{Bot, Channel, Database, Emoji, Message, Webhook};

/// Reference to some object in the database
#[derive(Serialize, Deserialize)]
pub struct Reference {
    /// Id of object
    pub id: String,
}

impl Reference {
    /// Create a Ref from an unchecked string
    pub fn from_unchecked(id: String) -> Reference {
        Reference { id }
    }

    /// Fetch bot from Ref
    pub async fn as_bot(&self, db: &Database) -> Result<Bot> {
        db.fetch_bot(&self.id).await
    }

    /// Fetch emoji from Ref
    pub async fn as_emoji(&self, db: &Database) -> Result<Emoji> {
        db.fetch_emoji(&self.id).await
    }

    /// Fetch channel from Ref
    pub async fn as_channel(&self, db: &Database) -> Result<Channel> {
        db.fetch_channel(&self.id).await
    }

    /// Fetch message from Ref
    pub async fn as_message(&self, db: &Database) -> Result<Message> {
        db.fetch_message(&self.id).await
    }

    /// Fetch webhook from Ref
    pub async fn as_webhook(&self, db: &Database) -> Result<Webhook> {
        db.fetch_webhook(&self.id).await
    }
}

#[cfg(feature = "rocket-impl")]
impl<'r> FromParam<'r> for Reference {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(Reference::from_unchecked(param.into()))
    }
}

#[cfg(feature = "rocket-impl")]
impl JsonSchema for Reference {
    fn schema_name() -> String {
        "Id".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            ..Default::default()
        })
    }
}
