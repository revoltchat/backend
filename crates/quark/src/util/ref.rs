use futures::future::join;
use rocket::request::FromParam;
use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::{Bot, Channel, Emoji, Invite, Member, Message, Server, ServerBan, User};
use crate::presence::presence_is_online;
use crate::{Database, Error, Result};

/// Reference to some object in the database
#[derive(Serialize, Deserialize)]
pub struct Ref {
    /// Id of object
    pub id: String,
}

impl Ref {
    /// Create a Ref from an unchecked string
    pub fn from_unchecked(id: String) -> Ref {
        Ref { id }
    }

    /// Fetch user from Ref
    pub async fn as_user(&self, db: &Database) -> Result<User> {
        let (user, online) = join(db.fetch_user(&self.id), presence_is_online(&self.id)).await;
        let mut user = user?;
        user.online = Some(online);
        Ok(user)
    }

    /// Fetch channel from Ref
    pub async fn as_channel(&self, db: &Database) -> Result<Channel> {
        db.fetch_channel(&self.id).await
    }

    /// Fetch server from Ref
    pub async fn as_server(&self, db: &Database) -> Result<Server> {
        db.fetch_server(&self.id).await
    }

    /// Fetch message from Ref
    pub async fn as_message(&self, db: &Database) -> Result<Message> {
        db.fetch_message(&self.id).await
    }

    /// Fetch message in channel from Ref
    pub async fn as_message_in(&self, db: &Database, channel: &str) -> Result<Message> {
        let message = self.as_message(db).await?;
        if message.channel != channel {
            return Err(Error::NotFound);
        }

        Ok(message)
    }

    /// Fetch bot from Ref
    pub async fn as_bot(&self, db: &Database) -> Result<Bot> {
        db.fetch_bot(&self.id).await
    }

    /// Fetch invite from Ref
    pub async fn as_invite(&self, db: &Database) -> Result<Invite> {
        Invite::find(db, &self.id).await
    }

    /// Fetch member from Ref
    pub async fn as_member(&self, db: &Database, server: &str) -> Result<Member> {
        db.fetch_member(server, &self.id).await
    }

    /// Fetch ban from Ref
    pub async fn as_ban(&self, db: &Database, server: &str) -> Result<ServerBan> {
        db.fetch_ban(server, &self.id).await
    }

    /// Fetch emoji from Ref
    pub async fn as_emoji(&self, db: &Database) -> Result<Emoji> {
        db.fetch_emoji(&self.id).await
    }
}

impl<'r> FromParam<'r> for Ref {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(Ref::from_unchecked(param.into()))
    }
}

impl JsonSchema for Ref {
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
