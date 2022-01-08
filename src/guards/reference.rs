use crate::*;

use rocket::request::FromParam;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct Ref {
    #[validate(length(min = 1, max = 26))]
    pub id: String,
}

impl Ref {
    pub fn from_unchecked(id: String) -> Ref {
        Ref { id }
    }

    pub fn from(id: String) -> Result<Ref> {
        let r = Ref { id };
        r.validate()
            .map_err(|error| Error::FailedValidation { error })?;
        Ok(r)
    }

    pub async fn fetch_user(&self) -> Result<User> {
        Ok(get_db().get_user(self.id.as_str()).await?)
    }

    pub async fn fetch_channel(&self) -> Result<Channel> {
        Ok(get_db().get_channel(self.id.as_str()).await?)
    }

    pub async fn fetch_server(&self) -> Result<Server> {
        Ok(get_db().get_server(self.id.as_str()).await?)
    }

    pub async fn fetch_invite(&self) -> Result<Invite> {
        Ok(get_db().get_invite(self.id.as_str()).await?)
    }

    pub async fn fetch_bot(&self) -> Result<Bot> {
        Ok(get_db().get_bot(self.id.as_str()).await?)
    }

    pub async fn fetch_member(&self, server: &str) -> Result<Member> {
        Ok(get_db().get_member(self.id.as_str(), server).await?)
    }

    pub async fn fetch_ban(&self, server: &str) -> Result<Ban> {
        Ok(get_db().get_ban(self.id.as_str(), server).await?)
    }

    pub async fn fetch_message(&self, channel: &Channel) -> Result<Message> {
        let message: Message = get_db().get_message(self.id.as_str()).await?;
        if &message.channel != channel.id() {
            Err(Error::InvalidOperation)
        } else {
            Ok(message)
        }
    }
}

impl User {
    pub fn as_ref(&self) -> Ref {
        Ref {
            id: self.id.to_string(),
        }
    }
}

impl<'r> FromParam<'r> for Ref {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if let Ok(result) = Ref::from(param.to_string()) {
            if result.validate().is_ok() {
                return Ok(result);
            }
        }

        Err(param)
    }
}
